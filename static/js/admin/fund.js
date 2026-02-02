/**
 * Fund Portfolio Management JavaScript
 *
 * Features:
 * 1. Multiple portfolios support
 * 2. Frontend editable table (fund_type, target_ratio, fund_name)
 * 3. Real-time calculation of ratios and allocations
 * 4. Smart incremental fund allocation by major category first
 * 5. Minor category subtotals
 * 6. Real-time calculation on input change (no save button for incremental/redemption)
 * 7. Redemption fund allocation
 * 8. Share portfolio via URL with LZString compression
 */

(function () {
  "use strict";

  // ============== URL Sharing Utilities ==============

  /**
   * Compress and encode portfolio data for URL sharing
   * @param {Array} entries - Array of fund entries
   * @param {Object} options - Additional options (incremental, redemption amounts)
   * @returns {string} Compressed and URL-safe encoded string
   */
  function compressPortfolioData(entries, options = {}) {
    // Create minimal data structure for sharing
    const shareData = {
      v: 1, // version for future compatibility
      n: options.name || "分享的组合",
      e: entries.map((entry) => ({
        m: entry.major_category, // major_category
        i: entry.minor_category || "", // minor_category
        t: entry.fund_type || "", // fund_type
        f: entry.fund_name, // fund_name
        r: Math.round(entry.target_ratio * 10000) / 10000, // target_ratio (keep precision)
        a: Math.round(entry.amount * 100) / 100, // amount
      })),
    };

    // Add optional fields
    if (options.incremental && options.incremental > 0) {
      shareData.inc = options.incremental;
    }
    if (options.redemption && options.redemption > 0) {
      shareData.red = options.redemption;
    }

    // Convert to JSON and compress
    const jsonStr = JSON.stringify(shareData);

    // Use LZString to compress and make URL-safe (access via window for global scope)
    if (typeof window.LZString !== "undefined" && window.LZString.compressToEncodedURIComponent) {
      return window.LZString.compressToEncodedURIComponent(jsonStr);
    }

    // Fallback: Base64 encode (less efficient)
    console.warn("LZString not available, using Base64 fallback");
    return btoa(encodeURIComponent(jsonStr));
  }

  /**
   * Decompress and decode portfolio data from URL parameter
   * @param {string} compressed - Compressed and encoded string
   * @returns {Object|null} Decompressed data object or null if failed
   */
  function decompressPortfolioData(compressed) {
    try {
      let jsonStr;

      // Try LZString decompression first (access via window for global scope)
      if (typeof window.LZString !== "undefined" && window.LZString.decompressFromEncodedURIComponent) {
        jsonStr = window.LZString.decompressFromEncodedURIComponent(compressed);
      }

      // Fallback to Base64 decode
      if (!jsonStr) {
        try {
          jsonStr = decodeURIComponent(atob(compressed));
        } catch (e) {
          console.warn("Base64 decode failed", e);
          return null;
        }
      }

      if (!jsonStr) return null;

      const data = JSON.parse(jsonStr);

      // Validate version and structure
      if (!data || !data.e || !Array.isArray(data.e)) {
        console.warn("Invalid share data structure");
        return null;
      }

      // Convert back to full entry format
      const entries = data.e.map((e, idx) => ({
        id: -(idx + 1), // Temporary negative IDs for local entries
        major_category: e.m,
        minor_category: e.i || null,
        fund_type: e.t || null,
        fund_name: e.f,
        target_ratio: e.r,
        amount: e.a,
        sort_index: idx,
      }));

      return {
        name: data.n || "分享的组合",
        entries: entries,
        incremental: data.inc || 0,
        redemption: data.red || 0,
      };
    } catch (e) {
      console.error("Failed to decompress portfolio data", e);
      return null;
    }
  }

  /**
   * Generate share URL for current portfolio
   * @returns {string} Full URL with compressed portfolio data
   */
  function generateShareUrl() {
    const options = {
      name: currentPortfolio
        ? currentPortfolio.portfolio.name
        : "分享的组合",
      incremental: parseFloat($("#input-incremental").val()) || 0,
      redemption: parseFloat($("#input-redemption").val()) || 0,
    };

    const compressed = compressPortfolioData(entries, options);
    const baseUrl = window.location.origin + "/fund";
    return `${baseUrl}?d=${compressed}`;
  }

  /**
   * Copy text to clipboard with fallback
   * @param {string} text - Text to copy
   * @returns {Promise<boolean>} Success status
   */
  async function copyToClipboard(text) {
    // Modern Clipboard API
    if (navigator.clipboard && window.isSecureContext) {
      try {
        await navigator.clipboard.writeText(text);
        return true;
      } catch (e) {
        console.warn("Clipboard API failed, trying fallback", e);
      }
    }

    // Fallback for older browsers or insecure context
    const textArea = document.createElement("textarea");
    textArea.value = text;
    textArea.style.position = "fixed";
    textArea.style.left = "-9999px";
    textArea.style.top = "-9999px";
    document.body.appendChild(textArea);
    textArea.focus();
    textArea.select();

    try {
      document.execCommand("copy");
      document.body.removeChild(textArea);
      return true;
    } catch (e) {
      console.error("Fallback copy failed", e);
      document.body.removeChild(textArea);
      return false;
    }
  }

  /**
   * Share current portfolio - generate URL and copy to clipboard
   */
  async function sharePortfolio() {
    if (entries.length === 0) {
      alert("没有数据可以分享，请先添加基金");
      return;
    }

    const url = generateShareUrl();

    // Check URL length (browsers typically support up to ~2000 chars, but we allow more)
    if (url.length > 8000) {
      alert(
        `分享链接过长 (${url.length} 字符)，部分浏览器可能无法正常打开。建议减少基金数量后重试。`,
      );
    }

    const success = await copyToClipboard(url);

    if (success) {
      // Show success feedback
      showShareToast("分享链接已复制到剪贴板！");
    } else {
      // Show URL in a modal for manual copy
      showShareModal(url);
    }
  }

  /**
   * Show a brief toast notification
   * @param {string} message - Toast message
   */
  function showShareToast(message) {
    // Remove existing toast if any
    $(".share-toast").remove();

    const toast = $(
      `<div class="share-toast" style="
        position: fixed;
        bottom: 30px;
        left: 50%;
        transform: translateX(-50%);
        background: #5cb85c;
        color: white;
        padding: 12px 24px;
        border-radius: 4px;
        box-shadow: 0 2px 8px rgba(0,0,0,0.2);
        z-index: 9999;
        font-weight: bold;
        animation: fadeInUp 0.3s ease;
      ">${message}</div>`,
    );

    $("body").append(toast);

    // Auto remove after 3 seconds
    setTimeout(() => {
      toast.fadeOut(300, function () {
        $(this).remove();
      });
    }, 3000);
  }

  /**
   * Show modal with URL for manual copy
   * @param {string} url - URL to display
   */
  function showShareModal(url) {
    // Remove existing modal if any
    $("#share-url-modal").remove();

    const modal = $(`
      <div id="share-url-modal" class="modal fade" tabindex="-1" role="dialog">
        <div class="modal-dialog" role="document">
          <div class="modal-content">
            <div class="modal-header">
              <button type="button" class="close" data-dismiss="modal"><span>&times;</span></button>
              <h4 class="modal-title">分享链接</h4>
            </div>
            <div class="modal-body">
              <p>请手动复制以下链接：</p>
              <textarea id="share-url-text" class="form-control" rows="4" readonly style="word-break: break-all;">${url}</textarea>
            </div>
            <div class="modal-footer">
              <button type="button" class="btn btn-primary" id="btn-copy-url">复制链接</button>
              <button type="button" class="btn btn-default" data-dismiss="modal">关闭</button>
            </div>
          </div>
        </div>
      </div>
    `);

    $("body").append(modal);

    modal.modal("show");

    // Select all text when clicking
    $("#share-url-text").on("click", function () {
      this.select();
    });

    // Copy button
    $("#btn-copy-url").on("click", async function () {
      const success = await copyToClipboard(url);
      if (success) {
        $(this).text("已复制！").addClass("btn-success").removeClass("btn-primary");
        setTimeout(() => {
          modal.modal("hide");
        }, 1000);
      }
    });

    // Clean up on close
    modal.on("hidden.bs.modal", function () {
      $(this).remove();
    });
  }

  /**
   * Load portfolio from URL parameters
   * @returns {boolean} True if data was loaded from URL
   */
  function loadFromUrlParams() {
    const urlParams = new URLSearchParams(window.location.search);
    const compressedData = urlParams.get("d");

    if (!compressedData) {
      return false;
    }

    const data = decompressPortfolioData(compressedData);

    if (!data) {
      console.warn("Failed to parse URL share data");
      return false;
    }

    // Load the shared data
    entries = data.entries;
    currentPortfolio = {
      portfolio: {
        id: null,
        name: data.name,
        description: "通过分享链接加载",
        total_amount: entries.reduce((sum, e) => sum + e.amount, 0),
      },
      entries: entries,
    };

    // Set incremental/redemption values
    if (data.incremental > 0) {
      $("#input-incremental").val(data.incremental);
    }
    if (data.redemption > 0) {
      $("#input-redemption").val(data.redemption);
    }

    // Update major order and render
    updateMajorOrderFromEntries();
    renderPortfolioInfo();
    renderTable();
    showPortfolioView();

    // Show loaded indicator
    showShareToast(`已加载分享的组合：${data.name}`);

    // Update page title
    document.title = `${data.name} - 基金投资组合`;

    return true;
  }

  // API endpoints
  const API = {
    listPortfolios: "/api/v1/fund/portfolios",
    getPortfolio: (id) => `/api/v1/fund/portfolio/${id}`,
    createPortfolio: "/api/v1/fund/portfolio",
    updatePortfolio: "/api/v1/fund/portfolio/update",
    deletePortfolio: (id) => `/api/v1/fund/portfolio/delete/${id}`,
    createEntry: "/api/v1/fund/entry",
    updateEntry: "/api/v1/fund/entry/update",
    deleteEntry: (id) => `/api/v1/fund/entry/delete/${id}`,
    batchUpdate: "/api/v1/fund/entries/batch-update",
    batchOrder: "/api/v1/fund/entries/batch-order",
  };

  // State
  let currentPortfolio = null;
  let entries = [];
  let pendingChanges = {};
  let deleteTarget = null;
  let deleteType = null;
  let entryRowCounter = 0;
  const IS_ADMIN =
    typeof window !== "undefined" &&
    (window.__is_admin === true ||
      window.__is_admin === "true" ||
      window.__is_admin === 1 ||
      window.__is_admin === "1");

  // Sort configuration
  let sortConfig = {
    majorOrder: ["股票", "债券", "大宗商品", "现金"], // Major category order
    entrySortBy: "default", // Entry sort by: default, target_ratio, amount, fund_name
    entrySortAsc: false, // Ascending / descending
  };

  // ============== Initialization ==============

  $(document).ready(function () {
    bindEvents();

    // Check if there's shared data in URL first (works for both admin and visitor)
    const loadedFromUrl = loadFromUrlParams();

    if (IS_ADMIN) {
      if (!loadedFromUrl) {
        loadPortfolios();
      } else {
        // Also load portfolios dropdown for admin, but don't auto-select
        loadPortfolios();
      }
    } else {
      // visitor: do not load DB portfolios, hide admin-only controls
      $("#btn-save-all, #btn-delete-portfolio").hide();
      $("#btn-new-portfolio, #btn-modal-create, #btn-confirm-delete").hide();
      // show visitor banner and ensure unsaved indicator is hidden
      $("#visitor-banner").show();
      $("#unsaved-indicator").removeClass("show");

      if (!loadedFromUrl) {
        // show UI so visitor can use tools locally
        entries = [];
        pendingChanges = {};
        updateMajorOrderFromEntries();
        renderTable();
        showPortfolioView();
      }
    }
  });

  function bindEvents() {
    // Portfolio selection
    $("#portfolio-select").on("change", function () {
      const id = $(this).val();
      if (id) {
        loadPortfolio(id);
      } else {
        clearPortfolioView();
      }
    });

    // New portfolio
    $("#btn-new-portfolio").on("click", () =>
      $("#new-portfolio-modal").modal("show"),
    );
    $("#btn-modal-create").on("click", createPortfolio);

    // Delete portfolio
    $("#btn-delete-portfolio").on("click", function () {
      if (currentPortfolio) {
        deleteTarget = currentPortfolio.portfolio.id;
        deleteType = "portfolio";
        $("#delete-confirm-text").text(
          `确定要删除投资组合 "${currentPortfolio.portfolio.name}" 吗？所有相关数据将被删除。`,
        );
        $("#delete-modal").modal("show");
      }
    });

    // Confirm delete
    $("#btn-confirm-delete").on("click", confirmDelete);

    // Real-time calculation on incremental/redemption input
    $("#input-incremental, #input-redemption").on("input", function () {
      recalculateAll();
    });

    // Add entry
    $("#btn-add-entry").on("click", function () {
      entryRowCounter = 0;
      $("#entry-rows-container").empty();
      addEntryRow(); // Add first row
      $("#add-entry-form").slideDown();
    });
    $("#btn-add-more-row").on("click", addEntryRow);
    $("#btn-cancel-add").on("click", () => $("#add-entry-form").slideUp());
    $("#btn-confirm-add-all").on("click", addAllNewEntries);

    // Save all changes
    $("#btn-save-all").on("click", saveAllChanges);

    // Share portfolio
    $("#btn-share").on("click", sharePortfolio);

    // Recalculate
    $("#btn-calculate").on("click", recalculateAll);

    // Sort controls
    $("#entry-sort-by").on("change", function () {
      sortConfig.entrySortBy = $(this).val();
      renderTable();
    });
    $("#entry-sort-asc").on("change", function () {
      sortConfig.entrySortAsc = $(this).is(":checked");
      renderTable();
    });
    $("#btn-move-major-up, #btn-move-major-down").on("click", function () {
      const direction = $(this).attr("id") === "btn-move-major-up" ? -1 : 1;
      const selectedMajor = $("#major-order-select").val();
      if (!selectedMajor) return;
      moveMajorCategory(selectedMajor, direction);
    });
  }

  // ============== Portfolio Management ==============

  function loadPortfolios() {
    $.get(API.listPortfolios, function (response) {
      if (response.status) {
        const select = $("#portfolio-select");
        select.find("option:not(:first)").remove();
        response.data.forEach((p) => {
          select.append(`<option value="${p.id}">${p.name}</option>`);
        });
      }
    });
  }

  function loadPortfolio(id) {
    $.get(API.getPortfolio(id), function (response) {
      if (response.status) {
        currentPortfolio = response.data;
        entries = currentPortfolio.entries;
        pendingChanges = {};

        // Update major category order config to include all majors actually present
        updateMajorOrderFromEntries();
        // Apply locally saved order if present
        loadEntryOrder();

        renderPortfolioInfo();
        renderTable();
        showPortfolioView();
      } else {
        alert("加载失败: " + response.data);
      }
    });
  }

  function createPortfolio() {
    if (!IS_ADMIN) {
      alert("只有管理员可以创建组合");
      return;
    }
    const name = $("#modal-portfolio-name").val().trim();
    const description = $("#modal-portfolio-desc").val().trim();

    if (!name) {
      alert("请输入组合名称");
      return;
    }

    $.ajax({
      url: API.createPortfolio,
      type: "POST",
      contentType: "application/json",
      data: JSON.stringify({ name, description: description || null }),
      success: function (response) {
        if (response.status) {
          $("#new-portfolio-modal").modal("hide");
          $("#modal-portfolio-name").val("");
          $("#modal-portfolio-desc").val("");
          loadPortfolios();
          setTimeout(() => {
            $("#portfolio-select").val(response.data).trigger("change");
          }, 500);
        } else {
          alert("创建失败: " + response.data);
        }
      },
    });
  }

  function confirmDelete() {
    if (!IS_ADMIN) {
      alert("只有管理员可以删除");
      deleteTarget = null;
      deleteType = null;
      $("#delete-modal").modal("hide");
      return;
    }
    if (deleteType === "portfolio" && deleteTarget) {
      $.post(API.deletePortfolio(deleteTarget), function (response) {
        if (response.status) {
          $("#delete-modal").modal("hide");
          loadPortfolios();
          clearPortfolioView();
          currentPortfolio = null;
        } else {
          alert("删除失败: " + response.data);
        }
      });
    } else if (deleteType === "entry" && deleteTarget) {
      $.post(API.deleteEntry(deleteTarget), function (response) {
        if (response.status) {
          $("#delete-modal").modal("hide");
          loadPortfolio(response.data);
        } else {
          alert("删除失败: " + response.data);
        }
      });
    }
    deleteTarget = null;
    deleteType = null;
  }

  // ============== Entry Management ==============

  function addEntryRow() {
    entryRowCounter++;
    const rowId = entryRowCounter;
    const rowHtml = `
            <div class="entry-row" data-row-id="${rowId}">
                <div class="form-group">
                    <label>大类资产 *</label>
                    <select class="new-major-category" style="width: 80px;">
                        <option value="股票">股票</option>
                        <option value="债券">债券</option>
                        <option value="大宗商品">大宗商品</option>
                        <option value="现金">现金</option>
                    </select>
                </div>
                <div class="form-group">
                    <label>小类</label>
                    <input type="text" class="new-minor-category" placeholder="如: A股" style="width: 80px;">
                </div>
                <div class="form-group">
                    <label>基金类别</label>
                    <input type="text" class="new-fund-type" placeholder="如: 沪深300" style="width: 100px;">
                </div>
                <div class="form-group">
                    <label>基金名称 *</label>
                    <input type="text" class="new-fund-name" placeholder="基金名称(代码)" style="width: 250px;">
                </div>
                <div class="form-group">
                    <label>计划比例(%)</label>
                    <input type="number" class="new-target-ratio" value="0" step="0.01" style="width: 70px;">
                </div>
                <div class="form-group">
                    <label>金额</label>
                    <input type="number" class="new-amount" value="0" step="0.01" style="width: 100px;">
                </div>
                <button type="button" class="btn btn-xs btn-danger btn-remove-row" data-row-id="${rowId}">
                    <span class="glyphicon glyphicon-minus"></span>
                </button>
            </div>
        `;
    $("#entry-rows-container").append(rowHtml);

    // Bind remove button
    $(`.btn-remove-row[data-row-id="${rowId}"]`).on("click", function () {
      $(`.entry-row[data-row-id="${rowId}"]`).remove();
    });
  }

  function addAllNewEntries() {
    // allow visitors to add entries locally (no backend calls)
    const rows = $("#entry-rows-container .entry-row");
    if (rows.length === 0) {
      alert("请至少添加一行");
      return;
    }
    const entriesToAdd = [];
    let hasError = false;

    rows.each(function () {
      const row = $(this);
      const fundName = row.find(".new-fund-name").val().trim();

      if (!fundName) {
        hasError = true;
        row.find(".new-fund-name").css("border-color", "red");
        return;
      }

      entriesToAdd.push({
        portfolio_id: currentPortfolio ? currentPortfolio.portfolio.id : null,
        major_category: row.find(".new-major-category").val(),
        minor_category: row.find(".new-minor-category").val().trim() || null,
        fund_type: row.find(".new-fund-type").val().trim() || null,
        fund_name: fundName,
        target_ratio:
          parseFloat(row.find(".new-target-ratio").val()) / 100 || 0,
        amount: parseFloat(row.find(".new-amount").val()) || 0,
      });
    });

    if (hasError) {
      alert("请填写所有基金名称");
      return;
    }

    if (!IS_ADMIN) {
      // local-only add for visitors: assign temporary negative ids
      if (!window.__localEntryIdCounter) window.__localEntryIdCounter = -1;
      entriesToAdd.forEach((entry) => {
        const localEntry = {
          id: window.__localEntryIdCounter--,
          portfolio_id: entry.portfolio_id,
          major_category: entry.major_category,
          minor_category: entry.minor_category,
          fund_type: entry.fund_type,
          fund_name: entry.fund_name,
          target_ratio: entry.target_ratio,
          amount: entry.amount,
          sort_index: entries.length,
        };
        entries.push(localEntry);
      });
      // close form and re-render
      $("#add-entry-form").slideUp();
      renderTable();
      return;
    }

    if (!currentPortfolio) return;

    // Use batch create API: send all new entries in one request
    $.ajax({
      url: API.createEntry,
      type: "POST",
      contentType: "application/json",
      data: JSON.stringify(entriesToAdd),
      success: function (response) {
        if (response && response.status) {
          $("#add-entry-form").slideUp();
          loadPortfolio(currentPortfolio.portfolio.id);
          alert("添加成功");
        } else {
          alert(
            "添加失败: " +
              (response && (response.data || response.error)
                ? response.data || response.error
                : ""),
          );
        }
      },
      error: function () {
        alert("添加失败");
        $("#add-entry-form").slideUp();
        if (currentPortfolio) loadPortfolio(currentPortfolio.portfolio.id);
      },
    });
  }

  function deleteEntry(id) {
    deleteTarget = id;
    deleteType = "entry";
    $("#delete-confirm-text").text("确定要删除这个基金条目吗？");
    $("#delete-modal").modal("show");
  }

  function updateEntryField(entryId, field, value) {
    const entry = entries.find((e) => e.id === entryId);
    if (!entry) return;

    // Track change only for admins (visitors shouldn't produce pendingChanges)
    if (IS_ADMIN) {
      if (!pendingChanges[entryId]) {
        pendingChanges[entryId] = {};
      }
      pendingChanges[entryId][field] = value;
    }

    // Update local state
    if (field === "target_ratio") {
      entry.target_ratio = value;
    } else if (field === "fund_type") {
      entry.fund_type = value;
    } else if (field === "fund_name") {
      entry.fund_name = value;
    } else if (field === "amount") {
      entry.amount = value;
    } else if (field === "minor_category") {
      entry.minor_category = value || null;
    }

    updateUnsavedIndicator();
    recalculateAll();
  }

  function saveAllChanges() {
    if (!IS_ADMIN) {
      alert("只有管理员可以保存更改");
      return;
    }
    const updates = [];

    // Collect all pending entry field changes
    Object.keys(pendingChanges).forEach((entryId) => {
      const changes = pendingChanges[entryId];
      if (Object.keys(changes).length > 0) {
        updates.push({
          id: parseInt(entryId),
          ...changes,
        });
      }
    });

    // Prepare order updates based on current entries order
    const orderUpdates = entries.map((e, idx) => ({
      id: e.id,
      sort_index: idx,
    }));

    if (updates.length === 0 && orderUpdates.length === 0) {
      alert("没有需要保存的更改");
      return;
    }

    // Validate: only when there are changes to save, ensure major ratios sum to 100% (allow small epsilon)
    try {
      const calculated = calculateAll();
      const sumMajorRatio = Object.keys(calculated.subtotals).reduce(
        (s, k) => s + (calculated.subtotals[k].targetRatio || 0),
        0,
      );
      const sumPercent = sumMajorRatio * 100;
      if (Math.abs(sumPercent - 100) > 0.01) {
        const msg = `大类计划比例合计为 ${sumPercent.toFixed(2)}%，必须为 100.00% 才能保存`;
        $("#major-ratio-error").text(msg).show();
        return;
      } else {
        $("#major-ratio-error").hide();
      }
    } catch (e) {
      console.warn("校验大类比例失败", e);
    }

    // We'll send updates in a single batch request, plus one request for the batch order if needed.
    let totalRequests =
      (updates.length > 0 ? 1 : 0) + (orderUpdates.length > 0 ? 1 : 0);
    let completed = 0;
    let failed = 0;

    function checkDone() {
      if (completed === totalRequests) {
        pendingChanges = {};
        updateUnsavedIndicator();
        loadPortfolio(currentPortfolio.portfolio.id);
        if (failed > 0) {
          alert(`保存完成，${failed} 条失败`);
        } else {
          alert("保存成功");
        }
      }
    }

    // Send a single batch update request for all changed entries
    if (updates.length > 0) {
      $.ajax({
        url: API.updateEntry,
        type: "POST",
        contentType: "application/json",
        data: JSON.stringify(updates),
        success: function (response) {
          if (!response.status) failed++;
          completed++;
          checkDone();
        },
        error: function () {
          failed++;
          completed++;
          checkDone();
        },
      });
    }

    // Send batch order once (backend will persist sort_index)
    if (orderUpdates.length > 0) {
      $.ajax({
        url: API.batchOrder,
        type: "POST",
        contentType: "application/json",
        data: JSON.stringify({
          portfolio_id: currentPortfolio.portfolio.id,
          updates: orderUpdates,
        }),
        success: function (response) {
          if (!response.status) failed++;
          completed++;
          checkDone();
        },
        error: function () {
          // fallback: still mark as completed but record failure
          failed++;
          completed++;
          checkDone();
        },
      });
    }
  }

  // ============== Unsaved Indicator ==============

  function updateUnsavedIndicator() {
    if (!IS_ADMIN) {
      // visitors don't get unsaved indicators
      $("#unsaved-indicator").removeClass("show");
      $("#btn-save-all").removeClass("has-changes");
      return;
    }
    const hasChanges = Object.keys(pendingChanges).some(
      (id) => Object.keys(pendingChanges[id]).length > 0,
    );

    if (hasChanges) {
      $("#unsaved-indicator").addClass("show");
      $("#btn-save-all").addClass("has-changes");
    } else {
      $("#unsaved-indicator").removeClass("show");
      $("#btn-save-all").removeClass("has-changes");
    }
  }

  // ============== Rendering ==============

  function showPortfolioView() {
    $(
      "#portfolio-info, #fund-inputs, #action-buttons, #allocation-section, #sort-controls",
    ).show();
    $("#btn-delete-portfolio").prop("disabled", false);
    updateMajorOrderSelect();
  }

  function clearPortfolioView() {
    $(
      "#portfolio-info, #fund-inputs, #action-buttons, #add-entry-form, #allocation-section, #sort-controls",
    ).hide();
    $("#btn-delete-portfolio").prop("disabled", true);
    $("#fund-table-body").html(`
        <tr>
          <td colspan="13" style="text-align: center; padding: 40px; color: #999;">
            请选择一个投资组合
          </td>
        </tr>
      `);
  }

  function renderPortfolioInfo() {
    const p = currentPortfolio.portfolio;
    $("#info-name").text(p.name);
    $("#info-total").text(formatCurrency(p.total_amount));
    // Incremental and redemption amounts are not read from backend; used only in frontend
    $("#input-incremental").val(0);
    $("#input-redemption").val(0);
  }

  function renderTable() {
    const tbody = $("#fund-table-body");
    tbody.empty();

    if (entries.length === 0) {
      tbody.html(`
                <tr>
            <td colspan="13" style="text-align: center; padding: 40px; color: #999;">
                        暂无数据，点击"添加基金"开始
                    </td>
                </tr>
            `);
      return;
    }

    // Calculate totals
    const calculated = calculateAll();

    // Group by major category, then by minor category
    const majorGroups = groupByMajorCategory(entries);

    Object.keys(majorGroups).forEach((majorCat) => {
      const majorEntries = majorGroups[majorCat];
      const minorGroups = groupByMinorCategory(majorEntries);
      const minorKeys = Object.keys(minorGroups);

      // Calculate total rows for a major (data rows + minor subtotal rows)
      let majorRowSpan = majorEntries.length;
      minorKeys.forEach((minorCat) => {
        if (minorCat !== "__none__" && minorGroups[minorCat].length > 1) {
          majorRowSpan += 1; // add minor subtotal row
        }
      });

      let isFirstInMajor = true;

      minorKeys.forEach((minorCat) => {
        const minorEntries = minorGroups[minorCat];
        const hasMinorSubtotal =
          minorCat !== "__none__" && minorEntries.length > 1;
        const minorRowSpan = minorEntries.length;
        let isFirstInMinor = true;

        minorEntries.forEach((entry, idx) => {
          const calc = calculated.entries[entry.id];
          const row = createEntryRow(
            entry,
            calc,
            isFirstInMajor,
            majorRowSpan,
            isFirstInMinor,
            minorRowSpan,
            entry.minor_category,
          );
          tbody.append(row);
          isFirstInMajor = false;
          isFirstInMinor = false;
        });

        // Minor category subtotal (only if more than 1 entry in this minor category)
        if (hasMinorSubtotal) {
          const minorSubtotal =
            calculated.minorSubtotals[majorCat + "|" + minorCat];
          if (minorSubtotal) {
            tbody.append(createMinorSubtotalRow(minorCat, minorSubtotal));
          }
        }
      });

      // Major category subtotal row
      const subtotal = calculated.subtotals[majorCat];
      tbody.append(createSubtotalRow(majorCat, subtotal));
    });

    // Total row
    tbody.append(createTotalRow(calculated.total));

    // Bind cell edit events
    bindCellEditEvents();
    // Enable drag & drop after binding events
    enableRowDragAndDrop();
  }

  // Drag & drop support for reordering entries
  function enableRowDragAndDrop() {
    const tbody = $("#fund-table-body");
    let dragSrcId = null;

    tbody.find("tr[data-id]").off("dragstart dragover dragleave drop dragend");

    tbody.find("tr[data-id]").on("dragstart", function (e) {
      dragSrcId = $(this).data("id");
      e.originalEvent.dataTransfer.effectAllowed = "move";
      e.originalEvent.dataTransfer.setData("text/plain", dragSrcId);
      $(this).addClass("dragging");
    });

    tbody.find("tr[data-id]").on("dragover", function (e) {
      e.preventDefault();
      $(this).addClass("drag-over");
    });

    tbody.find("tr[data-id]").on("dragleave", function () {
      $(this).removeClass("drag-over");
    });

    tbody.find("tr[data-id]").on("drop", function (e) {
      e.preventDefault();
      $(this).removeClass("drag-over");
      const srcId = e.originalEvent.dataTransfer.getData("text/plain");
      const targetId = $(this).data("id");
      if (!srcId || srcId === targetId) return;

      // Reorder entries array according to new DOM order
      const srcRow = tbody.find(`tr[data-id="${srcId}"]`);
      const targetRow = $(this);
      // Insert src before target
      targetRow.before(srcRow);

      // Rebuild entries order from DOM
      const newOrderIds = [];
      tbody.find("tr[data-id]").each(function () {
        newOrderIds.push($(this).data("id"));
      });

      // Reorder entries array to follow newOrderIds, preserving missing ones
      const idToEntry = {};
      entries.forEach((e) => (idToEntry[e.id] = e));
      const newEntries = [];
      newOrderIds.forEach((id) => {
        if (idToEntry[id]) newEntries.push(idToEntry[id]);
      });
      // Append any entries not present in DOM (shouldn't happen)
      entries.forEach((e) => {
        if (!newEntries.includes(e)) newEntries.push(e);
      });
      entries = newEntries;

      // Persist order for this portfolio
      saveEntryOrder();

      // Re-render to ensure grouping/rowspan correct
      renderTable();
    });

    tbody.find("tr[data-id]").on("dragend", function () {
      $(this).removeClass("dragging");
      tbody.find("tr").removeClass("drag-over");
    });
  }

  function saveEntryOrder() {
    if (!currentPortfolio) return;
    try {
      const key = `fund_order_${currentPortfolio.portfolio.id}`;
      const ids = entries.map((e) => e.id);
      // Only persist locally for now; actual backend submission happens on Save All
      localStorage.setItem(key, JSON.stringify(ids));
    } catch (e) {
      console.warn("保存排序到 localStorage 失败", e);
    }
  }

  function loadEntryOrder() {
    if (!currentPortfolio) return;
    try {
      const key = `fund_order_${currentPortfolio.portfolio.id}`;
      const raw = localStorage.getItem(key);
      if (!raw) return;
      const ids = JSON.parse(raw);
      if (!Array.isArray(ids)) return;
      const idToEntry = {};
      entries.forEach((e) => (idToEntry[e.id] = e));
      const reordered = [];
      ids.forEach((id) => {
        if (idToEntry[id]) reordered.push(idToEntry[id]);
      });
      // Append remaining entries not in saved order
      entries.forEach((e) => {
        if (!reordered.includes(e)) reordered.push(e);
      });
      entries = reordered;
    } catch (e) {
      console.warn("加载排序失败", e);
    }
  }

  function createEntryRow(
    entry,
    calc,
    isFirstInMajor,
    majorRowSpan,
    isFirstInMinor,
    minorRowSpan,
    minorCategory,
  ) {
    const deviationClass = calc.deviation >= 0 ? "positive" : "negative";
    const rebalanceClass = calc.rebalance >= 0 ? "positive" : "negative";
    const allocationDisplay =
      calc.allocation > 0 ? formatNumber(calc.allocation) : "";
    const redemptionDisplay =
      calc.redemption > 0 ? formatNumber(calc.redemption) : "";

    // Major category column: merged using rowspan
    const majorCategoryCell = isFirstInMajor
      ? `<td class="major-category" rowspan="${majorRowSpan}">${entry.major_category}</td>`
      : "";
    // Minor category column: merged for display using rowspan; expand for editing
    let minorCategoryCell = "";
    if (isFirstInMinor) {
      if (minorRowSpan > 1) {
        minorCategoryCell = `<td class="editable minor-merged" data-field="minor_category" data-minor-rowspan="${minorRowSpan}" rowspan="${minorRowSpan}">${minorCategory || ""}</td>`;
      } else {
        minorCategoryCell = `<td class="editable" data-field="minor_category">${minorCategory || ""}</td>`;
      }
    }
    // For non-first rows the minor cell is omitted (covered by rowspan); mark merged rows
    const isMergedMinor =
      !isFirstInMinor && minorRowSpan > 1 ? 'data-minor-merged="true"' : "";

    return `
            <tr draggable="true" data-id="${entry.id}" class="${isFirstInMajor ? "major-first-row" : ""}" ${isMergedMinor}>
                <td class="drag-handle" style="cursor: move; text-align: center;">☰</td>
                ${majorCategoryCell}
                ${minorCategoryCell}
                <td class="editable" data-field="fund_type">${entry.fund_type || ""}</td>
                <td class="editable" data-field="target_ratio">${formatPercent(entry.target_ratio)}</td>
                <td class="editable fund-name-cell" data-field="fund_name">${entry.fund_name}</td>
                <td>${formatPercent(calc.actualRatio)}</td>
                <td class="editable amount-cell" data-field="amount">${formatNumber(entry.amount)}</td>
                <td class="${deviationClass}">${formatPercent(calc.deviation)}</td>
                <td class="${rebalanceClass}">${formatNumber(calc.rebalance)}</td>
                <td class="${calc.allocation > 0 ? "positive" : ""}">${allocationDisplay}</td>
                <td>${redemptionDisplay}</td>
                <td>
                    <button class="btn btn-xs btn-danger btn-delete-entry" data-id="${entry.id}">
                        <span class="glyphicon glyphicon-trash"></span>
                    </button>
                </td>
            </tr>
        `;
  }

  function createMinorSubtotalRow(minorCat, subtotal) {
    const deviationClass = subtotal.deviation >= 0 ? "positive" : "negative";
    const rebalanceClass = subtotal.rebalance >= 0 ? "positive" : "negative";
    const allocationDisplay =
      subtotal.allocation > 0 ? formatNumber(subtotal.allocation) : "";
    const redemptionDisplay =
      subtotal.redemption > 0 ? formatNumber(subtotal.redemption) : "";
    return `
        <tr class="minor-subtotal-row">
          <td></td>
          <td colspan="2" style="text-align: center;">${minorCat} 小计</td>
                <td>${formatPercent(subtotal.targetRatio)}</td>
                <td></td>
                <td>${formatPercent(subtotal.actualRatio)}</td>
                <td>${formatNumber(subtotal.amount)}</td>
                <td class="${deviationClass}">${formatPercent(subtotal.deviation)}</td>
                <td class="${rebalanceClass}">${formatNumber(subtotal.rebalance)}</td>
                <td>${allocationDisplay}</td>
                <td>${redemptionDisplay}</td>
                <td></td>
            </tr>
        `;
  }

  function createSubtotalRow(majorCat, subtotal) {
    const deviationClass = subtotal.deviation >= 0 ? "positive" : "negative";
    const allocationDisplay =
      subtotal.allocation > 0 ? formatNumber(subtotal.allocation) : "";
    const redemptionDisplay =
      subtotal.redemption > 0 ? formatNumber(subtotal.redemption) : "";
    return `
        <tr class="subtotal-row">
          <td></td>
          <td colspan="3" style="text-align: left; padding-left: 10px;">${majorCat} 小计</td>
                <td>${formatPercent(subtotal.targetRatio)}</td>
                <td></td>
                <td>${formatPercent(subtotal.actualRatio)}</td>
                <td>${formatNumber(subtotal.amount)}</td>
                <td class="${deviationClass}">${formatPercent(subtotal.deviation)}</td>
                <td class="${subtotal.rebalance >= 0 ? "positive" : "negative"}">${formatNumber(subtotal.rebalance)}</td>
                <td>${allocationDisplay}</td>
                <td>${redemptionDisplay}</td>
                <td></td>
            </tr>
        `;
  }

  function createTotalRow(total) {
    const allocationDisplay =
      total.allocation > 0 ? formatNumber(total.allocation) : "";
    const redemptionDisplay =
      total.redemption > 0 ? formatNumber(total.redemption) : "";
    return `
        <tr class="total-row">
          <td></td>
          <td colspan="4" style="text-align: left; padding-left: 10px;">总计</td>
                <td></td>
                <td>100.00%</td>
                <td>${formatNumber(total.amount)}</td>
                <td>${formatPercent(total.deviationPercent)}</td>
                <td></td>
                <td>${allocationDisplay}</td>
                <td>${redemptionDisplay}</td>
                <td></td>
            </tr>
        `;
  }

  function bindCellEditEvents() {
    // Editable cell click
    $(".editable").on("click", function () {
      const cell = $(this);
      if (cell.find("input").length) return;

      const row = cell.closest("tr");
      const id = row.data("id");
      const field = cell.data("field");
      const entry = entries.find((e) => e.id === id);
      if (!entry) return;

      // If this is a merged minor-category cell, expand it first
      if (field === "minor_category" && cell.hasClass("minor-merged")) {
        expandMinorCells(cell);
        return; // after expansion events are rebound; user can click again to edit
      }

      let currentValue;
      if (field === "target_ratio") {
        currentValue = (entry.target_ratio * 100).toFixed(2);
      } else if (field === "amount") {
        currentValue = entry.amount;
      } else {
        currentValue = entry[field] || "";
      }

      const inputType =
        field === "amount" || field === "target_ratio" ? "number" : "text";
      const step =
        field === "target_ratio" ? "0.01" : field === "amount" ? "0.01" : "";
      const input = $(
        `<input type="${inputType}" ${step ? `step="${step}"` : ""} value="${currentValue}">`,
      );
      cell.html(input);
      input.focus().select();

      input.on("blur", function () {
        let newValue = $(this).val();

        if (field === "target_ratio") {
          newValue = parseFloat(newValue) / 100 || 0;
        } else if (field === "amount") {
          newValue = parseFloat(newValue) || 0;
        } else {
          newValue = newValue.trim();
        }

        updateEntryField(id, field, newValue);

        // 小类编辑完成后，重新渲染表格以合并同类项
        if (field === "minor_category") {
          renderTable();
        }
      });

      input.on("keypress", function (e) {
        if (e.which === 13) $(this).blur();
      });
    });

    // Delete entry button
    $(".btn-delete-entry").on("click", function () {
      const id = $(this).data("id");
      deleteEntry(id);
    });
  }

  // 拆开合并的小类单元格，让每行都有独立的小类单元格可编辑
  function expandMinorCells(mergedCell) {
    const rowspan = parseInt(mergedCell.attr("data-minor-rowspan")) || 1;
    const minorValue = mergedCell.text();
    const firstRow = mergedCell.closest("tr");

    // Remove rowspan attribute and convert to a regular cell
    mergedCell.removeAttr("rowspan").removeClass("minor-merged");

    // Insert minor-category cells for subsequent rows that were merged
    let currentRow = firstRow.next("tr");
    let count = 1;
    while (currentRow.length && count < rowspan) {
      // Check if this row was part of the merged group (has data-minor-merged)
      if (currentRow.attr("data-minor-merged") === "true") {
        const entryId = currentRow.data("id");
        const entry = entries.find((e) => e.id === entryId);
        const cellValue = entry ? entry.minor_category || "" : minorValue;
        // Insert the minor-category cell after the major-category cell
        const majorCell = currentRow.find(".major-category");
        if (majorCell.length) {
          majorCell.after(
            `<td class="editable" data-field="minor_category">${cellValue}</td>`,
          );
        } else {
          currentRow.prepend(
            `<td class="editable" data-field="minor_category">${cellValue}</td>`,
          );
        }
        currentRow.removeAttr("data-minor-merged");
        count++;
      }
      currentRow = currentRow.next("tr");
    }

    // Rebind edit and click events
    bindCellEditEvents();
  }

  // ============== Calculations ==============

  // Helper: distribute integer cents among items proportionally by weight
  function distributeCentsProportionalByWeight(items, totalCents) {
    // items: [{id, weight}, ...]
    const result = {};
    items.forEach((it) => (result[it.id] = 0));
    const totalWeight = items.reduce(
      (s, it) => s + (it.weight > 0 ? it.weight : 0),
      0,
    );
    if (!totalWeight || totalWeight <= 0) return result;
    let allocated = 0;
    let largest = null;
    items.forEach((it) => {
      if (it.weight > 0) {
        const share = Math.round(totalCents * (it.weight / totalWeight) || 0);
        result[it.id] = share;
        allocated += share;
        if (!largest || it.weight > largest.weight) largest = it;
      }
    });
    const diff = totalCents - allocated;
    if (diff !== 0) {
      const targetId = largest ? largest.id : items[0] && items[0].id;
      if (targetId) result[targetId] = (result[targetId] || 0) + diff;
    }
    return result;
  }

  function calculateAll() {
    const totalAmount = entries.reduce((sum, e) => sum + e.amount, 0);
    const incrementalAmount = parseFloat($("#input-incremental").val()) || 0;
    const redemptionAmount = parseFloat($("#input-redemption").val()) || 0;
    const newTotalAmount = totalAmount + incrementalAmount - redemptionAmount;
    // Rebalance base: if there is no incremental deposit and no redemption,
    // rebalancing should be an internal redistribution based on the current totalAmount
    // (do not use newTotalAmount) so that the sum of rebalances equals 0.
    const rebalanceBase =
      incrementalAmount === 0 && redemptionAmount === 0
        ? totalAmount
        : newTotalAmount;

    const result = {
      entries: {},
      minorSubtotals: {},
      subtotals: {},
      total: {
        amount: totalAmount,
        allocation: 0,
        redemption: 0,
        deviationPercent: 0,
      },
    };

    // Calculate allocations by major category first
    const allocations = calculateAllocationByMajorCategory(
      entries,
      totalAmount,
      incrementalAmount,
    );
    const redemptions = calculateRedemptionByMajorCategory(
      entries,
      totalAmount,
      redemptionAmount,
    );

    // Calculate each entry
    // Use integer cents for rebalance to avoid rounding drift; store per-entry rebalance cents
    const rebalanceCentsById = {};
    entries.forEach((entry) => {
      const actualRatio = totalAmount > 0 ? entry.amount / totalAmount : 0;
      const deviation = entry.target_ratio - actualRatio;
      const deviationTotal =
        totalAmount > 0
          ? (entry.target_ratio * totalAmount - entry.amount) / totalAmount
          : 0;

      const targetAmountCents = Math.round(
        rebalanceBase * entry.target_ratio * 100,
      );
      const amountCents = Math.round(entry.amount * 100);
      const rebalanceCents = targetAmountCents - amountCents;
      rebalanceCentsById[entry.id] = rebalanceCents;

      result.entries[entry.id] = {
        actualRatio,
        deviation,
        deviationTotal,
        rebalance: rebalanceCents / 100,
        allocation: allocations[entry.id] || 0,
        redemption: redemptions[entry.id] || 0,
      };
    });

    // Calculate minor subtotals
    const majorGroups = groupByMajorCategory(entries);
    Object.keys(majorGroups).forEach((majorCat) => {
      const majorEntries = majorGroups[majorCat];
      const minorGroups = groupByMinorCategory(majorEntries);

      Object.keys(minorGroups).forEach((minorCat) => {
        if (minorCat === "__none__") return;
        const minorEntries = minorGroups[minorCat];
        if (minorEntries.length <= 1) return;

        const key = majorCat + "|" + minorCat;
        const minorAmount = minorEntries.reduce((sum, e) => sum + e.amount, 0);
        const minorTargetRatio = minorEntries.reduce(
          (sum, e) => sum + e.target_ratio,
          0,
        );
        const minorActualRatio =
          totalAmount > 0 ? minorAmount / totalAmount : 0;
        // rebalance computed by summing per-entry rebalance cents to ensure consistency
        const minorRebalanceCents = minorEntries.reduce(
          (s, e) => s + (rebalanceCentsById[e.id] || 0),
          0,
        );
        result.minorSubtotals[key] = {
          targetRatio: minorTargetRatio,
          amount: minorAmount,
          actualRatio: minorActualRatio,
          deviation: minorTargetRatio - minorActualRatio,
          rebalance: minorRebalanceCents / 100,
          allocation:
            minorEntries.reduce(
              (s, e) =>
                s + Math.round(((allocations && allocations[e.id]) || 0) * 100),
              0,
            ) / 100,
          redemption:
            minorEntries.reduce(
              (s, e) =>
                s + Math.round(((redemptions && redemptions[e.id]) || 0) * 100),
              0,
            ) / 100,
        };
      });
    });

    // Calculate major category subtotals
    Object.keys(majorGroups).forEach((majorCat) => {
      const groupEntries = majorGroups[majorCat];
      const subtotal = {
        targetRatio: groupEntries.reduce((sum, e) => sum + e.target_ratio, 0),
        amount: groupEntries.reduce((sum, e) => sum + e.amount, 0),
        allocation:
          groupEntries.reduce(
            (s, e) =>
              s + Math.round(((allocations && allocations[e.id]) || 0) * 100),
            0,
          ) / 100,
        redemption:
          groupEntries.reduce(
            (s, e) =>
              s + Math.round(((redemptions && redemptions[e.id]) || 0) * 100),
            0,
          ) / 100,
      };
      subtotal.actualRatio =
        totalAmount > 0 ? subtotal.amount / totalAmount : 0;
      subtotal.deviation = subtotal.targetRatio - subtotal.actualRatio;
      // subtotal rebalance computed from per-entry rebalance cents
      const subtotalRebalanceCents = groupEntries.reduce(
        (s, e) => s + (rebalanceCentsById[e.id] || 0),
        0,
      );
      subtotal.rebalance = subtotalRebalanceCents / 100;

      result.subtotals[majorCat] = subtotal;
    });

    // Total allocation and redemption (compute from per-entry cents to avoid float drift)
    const totalAllocationCents = entries.reduce(
      (s, e) => s + Math.round(((allocations && allocations[e.id]) || 0) * 100),
      0,
    );
    const totalRedemptionCents = entries.reduce(
      (s, e) => s + Math.round(((redemptions && redemptions[e.id]) || 0) * 100),
      0,
    );
    result.total.allocation = totalAllocationCents / 100;
    result.total.redemption = totalRedemptionCents / 100;

    // Total rebalance (sum of per-entry rebalance cents) — should be 0 for internal rebalance
    const totalRebalanceCents = Object.keys(rebalanceCentsById || {}).reduce(
      (s, k) => s + (rebalanceCentsById[k] || 0),
      0,
    );
    result.total.rebalance = totalRebalanceCents / 100;

    // Calculate overall deviation percentage (sum of absolute deviations by major category)
    let totalDeviation = 0;
    Object.keys(result.subtotals).forEach((majorCat) => {
      totalDeviation += Math.abs(result.subtotals[majorCat].deviation);
    });
    result.total.deviationPercent = totalDeviation;

    return result;
  }

  /**
   * Allocation algorithm by major category:
   * Goal: allocate incremental funds so each major category approaches its target ratio.
   * Algorithm:
   * 1. Compute new total = totalAmount + incrementalAmount
   * 2. Calculate each major's target amount based on its target ratio
   * 3. If a major's target > current amount, it needs allocation
   * 4. If target <= current amount, allocate nothing to that major
   */
  function calculateAllocationByMajorCategory(
    entries,
    totalAmount,
    incrementalAmount,
  ) {
    const result = {};
    entries.forEach((e) => (result[e.id] = 0));

    if (incrementalAmount <= 0 || totalAmount <= 0) return result;

    const newTotalAmount = totalAmount + incrementalAmount;
    const majorGroups = groupByMajorCategory(entries);

    // Step 1: 计算每个大类的当前金额、目标金额、需要增加的金额
    const majorData = {};
    let totalNeedAllocate = 0; // 所有需要增加的大类的增加金额之和

    Object.keys(majorGroups).forEach((majorCat) => {
      const majorEntries = majorGroups[majorCat];
      const targetRatio = majorEntries.reduce(
        (sum, e) => sum + e.target_ratio,
        0,
      );
      const currentAmount = majorEntries.reduce((sum, e) => sum + e.amount, 0);

      // 分配后的目标金额
      const targetAmountAfterAllocate = newTotalAmount * targetRatio;
      // 需要增加的金额 = 目标金额 - 当前金额
      const needAllocate = targetAmountAfterAllocate - currentAmount;

      majorData[majorCat] = {
        targetRatio,
        currentAmount,
        targetAmountAfterAllocate,
        needAllocate,
        entries: majorEntries,
      };

      // 只统计需要增加的大类（needAllocate > 0 表示当前持仓低于目标）
      if (needAllocate > 0) {
        totalNeedAllocate += needAllocate;
      }
    });

    // 如果没有需要增加的大类，则不分配
    if (totalNeedAllocate <= 0) return result;

    // Step 2: 按需要增加金额的比例分配增量资金（使用分为单位的整数运算避免浮点累计误差）
    const resultCents = {};
    // init
    entries.forEach((e) => (resultCents[e.id] = 0));

    const incrementalCents = Math.round(incrementalAmount * 100);

    Object.keys(majorData).forEach((majorCat) => {
      const data = majorData[majorCat];
      if (data.needAllocate <= 0) return;

      const majorAllocation =
        incrementalAmount * (data.needAllocate / totalNeedAllocate);
      const majorAllocationCents = Math.round(majorAllocation * 100);

      // build fundData for this major: use needAllocate as weight
      const localFunds = [];
      data.entries.forEach((entry) => {
        if (entry.target_ratio > 0) {
          const relativeTargetRatio = entry.target_ratio / data.targetRatio;
          const targetAmountAfterAllocate =
            (data.currentAmount + majorAllocation) * relativeTargetRatio;
          const needAllocate = targetAmountAfterAllocate - entry.amount;
          if (needAllocate > 0) {
            localFunds.push({ id: entry.id, weight: needAllocate });
          }
        }
      });

      if (localFunds.length <= 0) return;

      // distribute cents within major using shared helper
      const majorDistributed = distributeCentsProportionalByWeight(
        localFunds,
        majorAllocationCents,
      );
      Object.keys(majorDistributed).forEach((id) => {
        resultCents[id] = (resultCents[id] || 0) + majorDistributed[id];
      });
    });

    // global residual
    const allocatedTotalCents = Object.keys(resultCents).reduce(
      (s, k) => s + (resultCents[k] || 0),
      0,
    );
    const globalDiffCents = incrementalCents - allocatedTotalCents;
    if (globalDiffCents !== 0) {
      // absorb into the entry with largest allocated cents (or first entry)
      let targetId = null;
      let maxAllocated = -Infinity;
      Object.keys(resultCents).forEach((k) => {
        const v = resultCents[k] || 0;
        if (v > maxAllocated) {
          maxAllocated = v;
          targetId = k;
        }
      });
      if (!targetId && entries.length > 0) targetId = entries[0].id;
      if (targetId) {
        resultCents[targetId] = (resultCents[targetId] || 0) + globalDiffCents;
      }
    }

    // convert back to float yuan amounts
    Object.keys(resultCents).forEach((k) => {
      result[k] = (resultCents[k] || 0) / 100;
    });

    return result;
  }

  /**
   * Redemption algorithm by major category:
   * Goal: after redemption, adjust amounts so major categories approach their target ratios.
   * Algorithm: compute new total after redemption, then calculate how much to redeem
   * from each major to reach the target proportions.
   */
  function calculateRedemptionByMajorCategory(
    entries,
    totalAmount,
    redemptionAmount,
  ) {
    // We'll accumulate in cents to avoid per-branch rounding issues
    const result = {};
    const resultCents = {};
    entries.forEach((e) => {
      resultCents[e.id] = 0;
      result[e.id] = 0;
    });

    const newTotalAmount = totalAmount - redemptionAmount;
    if (newTotalAmount <= 0) {
      // 如果赎回后总额<=0，按持仓比例赎回（按分计算并吸收残差）
      const totalRoundedCents = Math.round(totalAmount * 100);
      let sumCents = 0;
      let largest = null;
      entries.forEach((entry) => {
        if (entry.amount > 0) {
          const cents = Math.round(entry.amount * 100);
          resultCents[entry.id] = cents;
          sumCents += cents;
          if (!largest || entry.amount > largest.amount) largest = entry;
        }
      });
      const diff = totalRoundedCents - sumCents;
      if (diff !== 0 && largest) {
        resultCents[largest.id] = (resultCents[largest.id] || 0) + diff;
      }
      Object.keys(resultCents).forEach((k) => {
        result[k] = (resultCents[k] || 0) / 100;
      });
      return result;
    }

    const majorGroups = groupByMajorCategory(entries);

    // Step 1: 计算每个大类的当前金额、目标比例、赎回后的目标金额
    const majorData = {};
    let totalNeedRedeem = 0; // 所有需要赎回的大类的赎回金额之和

    Object.keys(majorGroups).forEach((majorCat) => {
      const majorEntries = majorGroups[majorCat];
      const targetRatio = majorEntries.reduce(
        (sum, e) => sum + e.target_ratio,
        0,
      );
      const currentAmount = majorEntries.reduce((sum, e) => sum + e.amount, 0);

      // 赎回后的目标金额
      const targetAmountAfterRedeem = newTotalAmount * targetRatio;
      // 需要赎回的金额 = 当前金额 - 目标金额
      const needRedeem = currentAmount - targetAmountAfterRedeem;

      majorData[majorCat] = {
        targetRatio,
        currentAmount,
        targetAmountAfterRedeem,
        needRedeem,
        entries: majorEntries,
      };

      // 只统计需要赎回的大类（needRedeem > 0 表示当前持仓超过目标）
      if (needRedeem > 0) {
        totalNeedRedeem += needRedeem;
      }
    });

    // 如果没有需要赎回的大类，则按持仓比例赎回（按分分配）
    if (totalNeedRedeem <= 0) {
      Object.keys(majorGroups).forEach((majorCat) => {
        const majorEntries = majorGroups[majorCat];
        const majorAmount = majorEntries.reduce((sum, e) => sum + e.amount, 0);
        if (majorAmount <= 0) return;

        const majorRedemption = redemptionAmount * (majorAmount / totalAmount);
        const majorRedemptionCents = Math.round(majorRedemption * 100);
        const amtList = majorEntries
          .filter((e) => e.amount > 0)
          .map((e) => ({ id: e.id, weight: e.amount }));
        const distributed = distributeCentsProportionalByWeight(
          amtList,
          majorRedemptionCents,
        );
        Object.keys(distributed).forEach((id) => {
          resultCents[id] = (resultCents[id] || 0) + distributed[id];
        });
      });
      Object.keys(resultCents).forEach((k) => {
        result[k] = (resultCents[k] || 0) / 100;
      });
      return result;
    }

    // Step 2: 按需要赎回的比例分配赎回金额
    Object.keys(majorData).forEach((majorCat) => {
      const data = majorData[majorCat];

      // 跳过不需要赎回的大类
      if (data.needRedeem <= 0) return;

      // 按需要赎回的比例分配赎回金额
      const majorRedemption =
        redemptionAmount * (data.needRedeem / totalNeedRedeem);
      const majorRedemptionCents = Math.round(majorRedemption * 100);

      // 计算大类内赎回后的目标金额
      const majorAmountAfterRedeem = data.currentAmount - majorRedemption;

      // 计算各基金需要赎回的金额（按 needRedeem 为权重），若均无需要则按持仓比例
      const needList = [];
      const majorEntries = data.entries;
      majorEntries.forEach((entry) => {
        if (entry.amount > 0) {
          const relativeTargetRatio =
            data.targetRatio > 0 ? entry.target_ratio / data.targetRatio : 0;
          const targetAmountAfterRedeem =
            majorAmountAfterRedeem * relativeTargetRatio;
          const needRedeem = entry.amount - targetAmountAfterRedeem;
          if (needRedeem > 0) {
            needList.push({ id: entry.id, weight: needRedeem });
          }
        }
      });

      if (needList.length > 0) {
        const distributed = distributeCentsProportionalByWeight(
          needList,
          majorRedemptionCents,
        );
        Object.keys(distributed).forEach((id) => {
          resultCents[id] = (resultCents[id] || 0) + distributed[id];
        });
        return;
      }

      // fallback: 按持仓比例
      const amtList = data.entries
        .filter((e) => e.amount > 0)
        .map((e) => ({ id: e.id, weight: e.amount }));
      const distributed2 = distributeCentsProportionalByWeight(
        amtList,
        majorRedemptionCents,
      );
      Object.keys(distributed2).forEach((id) => {
        resultCents[id] = (resultCents[id] || 0) + distributed2[id];
      });
    });
    // 全局残差吸收：确保分配总和等于输入的赎回金额（以分为单位）
    const redemptionTotalCents = Math.round(redemptionAmount * 100);
    const allocatedTotalCents = Object.keys(resultCents).reduce(
      (s, k) => s + (resultCents[k] || 0),
      0,
    );
    const globalDiff = redemptionTotalCents - allocatedTotalCents;
    if (globalDiff !== 0) {
      // 吸收到持仓最多的条目
      let targetId = null;
      let maxAmount = -Infinity;
      entries.forEach((e) => {
        if (e.amount > maxAmount) {
          maxAmount = e.amount;
          targetId = e.id;
        }
      });
      if (!targetId && entries.length > 0) targetId = entries[0].id;
      if (targetId)
        resultCents[targetId] = (resultCents[targetId] || 0) + globalDiff;
    }

    // final convert
    Object.keys(resultCents).forEach((k) => {
      result[k] = (resultCents[k] || 0) / 100;
    });
    return result;
  }

  function recalculateAll() {
    // Allow recalculation even when no backend portfolio is loaded (visitor local mode)
    renderTable();
  }

  // ============== Utilities ==============

  function groupByMajorCategory(entries) {
    const groups = {};
    entries.forEach((entry) => {
      const cat = entry.major_category;
      if (!groups[cat]) groups[cat] = [];
      groups[cat].push(entry);
    });

    // Order majors according to configured `sortConfig.majorOrder`
    const orderedGroups = {};
    sortConfig.majorOrder.forEach((cat) => {
      if (groups[cat]) {
        orderedGroups[cat] = groups[cat];
      }
    });
    // 添加不在配置中的大类
    Object.keys(groups).forEach((cat) => {
      if (!orderedGroups[cat]) {
        orderedGroups[cat] = groups[cat];
      }
    });

    return orderedGroups;
  }

  function groupByMinorCategory(entries) {
    const groups = {};
    entries.forEach((entry) => {
      const cat = entry.minor_category || "__none__";
      if (!groups[cat]) groups[cat] = [];
      groups[cat].push(entry);
    });

    // Sort entries within each minor category according to current sort settings
    Object.keys(groups).forEach((cat) => {
      groups[cat] = sortEntries(groups[cat]);
    });

    return groups;
  }

  // Entry sorting function
  function sortEntries(entries) {
    if (sortConfig.entrySortBy === "default") {
      return entries;
    }

    const sorted = [...entries].sort((a, b) => {
      let valA, valB;
      switch (sortConfig.entrySortBy) {
        case "target_ratio":
          valA = a.target_ratio;
          valB = b.target_ratio;
          break;
        case "amount":
          valA = a.amount;
          valB = b.amount;
          break;
        case "fund_name":
          valA = a.fund_name || "";
          valB = b.fund_name || "";
          return sortConfig.entrySortAsc
            ? valA.localeCompare(valB)
            : valB.localeCompare(valA);
        default:
          return 0;
      }
      return sortConfig.entrySortAsc ? valA - valB : valB - valA;
    });

    return sorted;
  }

  // Move major category order up or down
  function moveMajorCategory(majorCat, direction) {
    const idx = sortConfig.majorOrder.indexOf(majorCat);
    if (idx === -1) return;

    const newIdx = idx + direction;
    if (newIdx < 0 || newIdx >= sortConfig.majorOrder.length) return;

    // 交换位置
    [sortConfig.majorOrder[idx], sortConfig.majorOrder[newIdx]] = [
      sortConfig.majorOrder[newIdx],
      sortConfig.majorOrder[idx],
    ];

    updateMajorOrderSelect();
    renderTable();
  }

  // Update the major-order select dropdown UI
  function updateMajorOrderSelect() {
    const select = $("#major-order-select");
    const currentVal = select.val();
    select.empty();
    sortConfig.majorOrder.forEach((cat) => {
      select.append(`<option value="${cat}">${cat}</option>`);
    });
    if (currentVal && sortConfig.majorOrder.includes(currentVal)) {
      select.val(currentVal);
    }
  }

  // Update `sortConfig.majorOrder` based on majors currently present in entries
  function updateMajorOrderFromEntries() {
    // 获取所有实际存在的大类
    const actualMajors = new Set();
    entries.forEach((e) => {
      if (e && e.major_category) actualMajors.add(e.major_category);
    });

    // 保留已有顺序中存在的大类
    const newOrder = sortConfig.majorOrder.filter((cat) =>
      actualMajors.has(cat),
    );

    // 添加新出现的大类（不在已有顺序中的）
    actualMajors.forEach((cat) => {
      if (!newOrder.includes(cat)) {
        newOrder.push(cat);
      }
    });

    sortConfig.majorOrder = newOrder;
  }

  function formatCurrency(value) {
    if (value === 0) return "¥0.00";
    const prefix = value < 0 ? "-" : "";
    const formatted = Math.abs(value)
      .toFixed(2)
      .replace(/\B(?=(\d{3})+(?!\d))/g, ",");
    return `${prefix}¥${formatted}`;
  }
  // Format a number to two decimals with thousand separators (no currency symbol)
  function formatNumber(value) {
    if (value === 0) return "0.00";
    const prefix = value < 0 ? "-" : "";
    const formatted = Math.abs(value)
      .toFixed(2)
      .replace(/\B(?=(\d{3})+(?!\d))/g, ",");
    return `${prefix}${formatted}`;
  }

  // Format a decimal ratio as percentage with two decimals
  function formatPercent(value) {
    return (value * 100).toFixed(2) + "%";
  }
})();
