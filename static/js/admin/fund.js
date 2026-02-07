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
  const calcCore = typeof window !== "undefined" ? window.FundCalcCore : null;
  if (!calcCore) {
    throw new Error("FundCalcCore is required before loading fund.js");
  }

  // ============== Bootstrap 5 Modal Helpers ==============
  // Cache modal instances for reuse
  const modalInstances = {};

  function getModal(selector) {
    const el = document.querySelector(selector);
    if (!el) return null;
    if (!modalInstances[selector]) {
      modalInstances[selector] = new bootstrap.Modal(el);
    }
    return modalInstances[selector];
  }

  function showModal(selector) {
    const modal = getModal(selector);
    if (modal) modal.show();
  }

  function hideModal(selector) {
    const modal = getModal(selector);
    if (modal) modal.hide();
  }

  // ============== URL Sharing Utilities ==============

  // Major category encoding
  const MAJOR_CATEGORY_MAP = { 股票: 0, 债券: 1, 大宗商品: 2, 现金: 3 };
  const MAJOR_CATEGORY_REVERSE = ["股票", "债券", "大宗商品", "现金"];

  // Base62 encoding for compact numbers (0-9a-zA-Z)
  const B62 = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
  function toB62(n) {
    if (n === 0) return "0";
    let s = "";
    while (n > 0) {
      s = B62[n % 62] + s;
      n = Math.floor(n / 62);
    }
    return s;
  }
  function fromB62(s) {
    let n = 0;
    for (const c of s) n = n * 62 + B62.indexOf(c);
    return n;
  }

  /**
   * Ultra-compact format:
   * [inc~red~]major~minor~type~name~ratio~amount;...
   * - Uses Base62 for numbers
   * - '~' as field separator, ';' as entry separator
   * - Header omitted if both 0
   * - Empty minor/type encoded as empty string
   */
  function compressPortfolioData(entries, options = {}) {
    const inc = Math.round(options.incremental || 0);
    const red = Math.round(options.redemption || 0);

    const parts = [];

    // Header only if needed
    if (inc > 0 || red > 0) {
      parts.push(`${toB62(inc)}~${toB62(red)}~`);
    }

    // Entries
    const entryStrs = entries.map((entry) => {
      const major = MAJOR_CATEGORY_MAP[entry.major_category] ?? 0;
      const minor = (entry.minor_category || "").replace(/[~;]/g, "");
      const type = (entry.fund_type || "").replace(/[~;]/g, "");
      const name = (entry.fund_name || "").replace(/[~;]/g, "");
      const ratio = toB62(Math.round(entry.target_ratio * 10000));
      const amount = toB62(Math.round(entry.amount));
      return `${major}~${minor}~${type}~${name}~${ratio}~${amount}`;
    });

    const str = parts.join("") + entryStrs.join(";");

    if (typeof window.LZString !== "undefined") {
      return window.LZString.compressToEncodedURIComponent(str);
    }
    return btoa(encodeURIComponent(str));
  }

  /**
   * Decompress portfolio data
   */
  function decompressPortfolioData(compressed) {
    try {
      let str;
      if (typeof window.LZString !== "undefined") {
        str = window.LZString.decompressFromEncodedURIComponent(compressed);
      }
      if (!str) return null;

      let incremental = 0,
        redemption = 0;
      let entriesStr = str;

      // Check for header (starts with number~number~, no ';' before first entry)
      const headerMatch = str.match(/^([0-9a-zA-Z]+)~([0-9a-zA-Z]+)~(?=\d)/);
      if (headerMatch) {
        incremental = fromB62(headerMatch[1]);
        redemption = fromB62(headerMatch[2]);
        entriesStr = str.slice(headerMatch[0].length);
      }

      // Parse entries
      const entries = [];
      const entryParts = entriesStr.split(";");

      for (let i = 0; i < entryParts.length; i++) {
        const fields = entryParts[i].split("~");
        if (fields.length < 6) continue;

        entries.push({
          id: -(i + 1),
          major_category: MAJOR_CATEGORY_REVERSE[parseInt(fields[0])] || "股票",
          minor_category: fields[1] || null,
          fund_type: fields[2] || null,
          fund_name: fields[3] || "",
          target_ratio: fromB62(fields[4]) / 10000,
          amount: fromB62(fields[5]),
          sort_index: i,
        });
      }

      return {
        name: "分享的组合",
        entries: entries,
        incremental: incremental,
        redemption: redemption,
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
      name: currentPortfolio ? currentPortfolio.portfolio.name : "分享的组合",
      incremental: parseMoneyInput($("#input-incremental").val()),
      redemption: parseMoneyInput($("#input-redemption").val()),
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

    const toast = $('<div class="share-toast"><span class="share-toast-text"></span></div>');
    toast.find(".share-toast-text").text(message);

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
              <h4 class="modal-title">分享链接</h4>
              <button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
            </div>
            <div class="modal-body">
              <p>请手动复制以下链接：</p>
              <textarea id="share-url-text" class="form-control" rows="4" readonly style="word-break: break-all;">${url}</textarea>
            </div>
            <div class="modal-footer">
              <button type="button" class="btn btn-primary" id="btn-copy-url">复制链接</button>
              <button type="button" class="btn btn-secondary" data-bs-dismiss="modal">关闭</button>
            </div>
          </div>
        </div>
      </div>
    `);

    $("body").append(modal);

    const bsModal = new bootstrap.Modal(modal[0]);
    bsModal.show();

    // Select all text when clicking
    $("#share-url-text").on("click", function () {
      this.select();
    });

    // Copy button
    $("#btn-copy-url").on("click", async function () {
      const success = await copyToClipboard(url);
      if (success) {
        $(this)
          .text("已复制！")
          .addClass("btn-success")
          .removeClass("btn-primary");
        setTimeout(() => {
          bsModal.hide();
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
    loadMoneyUnitForCurrentPortfolio();

    // Update major order and render
    updateMajorOrderFromEntries();
    renderPortfolioInfo();
    if (data.incremental > 0) {
      $("#input-incremental").val(formatMoneyInputValue(data.incremental));
    }
    if (data.redemption > 0) {
      $("#input-redemption").val(formatMoneyInputValue(data.redemption));
    }
    renderTable();
    showPortfolioView();

    // Show loaded indicator
    if (data.name && data.name !== "分享的组合") {
      showShareToast(`已加载分享组合：${data.name}`);
    } else {
      showShareToast("已加载分享组合");
    }

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
  let pendingNewEntries = []; // 本地新增的条目（尚未保存到数据库）
  let pendingDeleteIds = []; // 本地标记删除的条目ID（尚未从数据库删除）
  let deleteTarget = null;
  let deleteType = null;
  let entryRowCounter = 0;
  let pendingCellEditFocus = null;
  let moneyUnit = "¥";
  const SUPPORTED_MONEY_UNITS = ["¥", "$", "€", "£"];
  const DEFAULT_MONEY_UNIT = "¥";
  const MONEY_UNIT_STORAGE_PREFIX = "fund_money_unit_portfolio_";
  const MONEY_UNIT_STORAGE_SHARED_KEY = "fund_money_unit_shared";
  const MONEY_UNIT_STORAGE_LEGACY_KEY = "fund_money_unit";
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
    initMoneyUnit();
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
      showModal("#new-portfolio-modal"),
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
        showModal("#delete-modal");
      }
    });

    // Confirm delete
    $("#btn-confirm-delete").on("click", confirmDelete);

    // Real-time calculation on incremental/redemption input
    $("#input-incremental, #input-redemption")
      .on("input", function () {
        recalculateAll();
      })
      .on("focus", function () {
        this.select();
      })
      .on("blur", function () {
        $(this).val(formatMoneyInputValue($(this).val()));
        recalculateAll();
      });

    $("#money-unit-select").on("change", function () {
      setMoneyUnit($(this).val());
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
        pendingNewEntries = [];
        pendingDeleteIds = [];

        // Update major category order config to include all majors actually present
        updateMajorOrderFromEntries();
        // Apply locally saved order if present
        loadEntryOrder();
        loadMoneyUnitForCurrentPortfolio();

        renderPortfolioInfo();
        renderTable();
        showPortfolioView();
        updateUnsavedIndicator();
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
          hideModal("#new-portfolio-modal");
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
      hideModal("#delete-modal");
      return;
    }
    if (deleteType === "portfolio" && deleteTarget) {
      $.post(API.deletePortfolio(deleteTarget), function (response) {
        if (response.status) {
          hideModal("#delete-modal");
          loadPortfolios();
          clearPortfolioView();
          currentPortfolio = null;
        } else {
          alert("删除失败: " + response.data);
        }
      });
    } else if (deleteType === "entry" && deleteTarget) {
      // 本地删除条目，不立即调用API
      const entryId = deleteTarget;

      // 检查是否是本地新增的条目（负数ID）
      const newEntryIndex = pendingNewEntries.findIndex(
        (e) => e.id === entryId,
      );
      if (newEntryIndex !== -1) {
        // 从本地新增列表中移除
        pendingNewEntries.splice(newEntryIndex, 1);
      } else {
        // 已存在于数据库的条目，标记为待删除
        if (!pendingDeleteIds.includes(entryId)) {
          pendingDeleteIds.push(entryId);
        }
      }

      // 从entries数组中移除
      entries = entries.filter((e) => e.id !== entryId);

      // 清除该条目的pendingChanges
      delete pendingChanges[entryId];

      hideModal("#delete-modal");
      renderTable();
      updateUnsavedIndicator();
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
                    <input type="text" class="new-amount" value="0" inputmode="decimal" autocomplete="off" style="width: 100px;">
                </div>
                <button type="button" class="btn btn-sm btn-danger btn-remove-row" data-row-id="${rowId}">
                    <i class="bi bi-dash"></i>
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
        amount: parseMoneyInput(row.find(".new-amount").val()),
      });
    });

    if (hasError) {
      alert("请填写所有基金名称");
      return;
    }

    // 本地添加条目，不立即调用API（管理员和访客都使用本地添加）
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
      // 管理员模式下，跟踪本地新增的条目
      if (IS_ADMIN) {
        pendingNewEntries.push(localEntry);
      }
    });
    // close form and re-render
    $("#add-entry-form").slideUp();
    renderTable();
    updateUnsavedIndicator();
  }

  function deleteEntry(id) {
    deleteTarget = id;
    deleteType = "entry";
    $("#delete-confirm-text").text("确定要删除这个基金条目吗？");
    showModal("#delete-modal");
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

    // Collect all pending entry field changes (only for existing entries, not new ones)
    Object.keys(pendingChanges).forEach((entryId) => {
      const id = parseInt(entryId);
      // 跳过本地新增的条目（负数ID），它们会通过创建API处理
      if (id < 0) return;
      const changes = pendingChanges[entryId];
      if (Object.keys(changes).length > 0) {
        updates.push({
          id: id,
          ...changes,
        });
      }
    });

    // 准备要创建的新条目
    const entriesToCreate = pendingNewEntries.map((entry) => ({
      portfolio_id: currentPortfolio.portfolio.id,
      major_category: entry.major_category,
      minor_category: entry.minor_category,
      fund_type: entry.fund_type,
      fund_name: entry.fund_name,
      target_ratio: entry.target_ratio,
      amount: entry.amount,
    }));

    // 准备要删除的条目ID
    const idsToDelete = [...pendingDeleteIds];

    // Prepare order updates based on current entries order (only for existing entries)
    const orderUpdates = entries
      .filter((e) => e.id > 0) // 只包含已存在于数据库的条目
      .map((e, idx) => ({
        id: e.id,
        sort_index: idx,
      }));

    const hasFieldUpdates = updates.length > 0;
    const hasNewEntries = entriesToCreate.length > 0;
    const hasDeletes = idsToDelete.length > 0;
    const hasOrderUpdates = orderUpdates.length > 0;

    if (!hasFieldUpdates && !hasNewEntries && !hasDeletes && !hasOrderUpdates) {
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

    // 计算总请求数
    let totalRequests =
      (hasFieldUpdates ? 1 : 0) +
      (hasNewEntries ? 1 : 0) +
      (hasDeletes ? idsToDelete.length : 0) + // 每个删除一个请求
      (hasOrderUpdates ? 1 : 0);
    let completed = 0;
    let failed = 0;

    function checkDone() {
      if (completed === totalRequests) {
        pendingChanges = {};
        pendingNewEntries = [];
        pendingDeleteIds = [];
        updateUnsavedIndicator();
        loadPortfolio(currentPortfolio.portfolio.id);
        if (failed > 0) {
          alert(`保存完成，${failed} 条失败`);
        } else {
          alert("保存成功");
        }
      }
    }

    // 如果没有请求需要发送，直接返回
    if (totalRequests === 0) {
      alert("没有需要保存的更改");
      return;
    }

    // 1. 发送删除请求
    if (hasDeletes) {
      idsToDelete.forEach((id) => {
        $.post(API.deleteEntry(id), function (response) {
          if (!response.status) failed++;
          completed++;
          checkDone();
        }).fail(function () {
          failed++;
          completed++;
          checkDone();
        });
      });
    }

    // 2. 发送创建新条目请求
    if (hasNewEntries) {
      $.ajax({
        url: API.createEntry,
        type: "POST",
        contentType: "application/json",
        data: JSON.stringify(entriesToCreate),
        success: function (response) {
          if (!response || !response.status) failed++;
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

    // 3. Send a single batch update request for all changed entries
    if (hasFieldUpdates) {
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

    // 4. Send batch order once (backend will persist sort_index)
    if (hasOrderUpdates) {
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
    const hasFieldChanges = Object.keys(pendingChanges).some(
      (id) => Object.keys(pendingChanges[id]).length > 0,
    );
    const hasNewEntries = pendingNewEntries.length > 0;
    const hasDeletedEntries = pendingDeleteIds.length > 0;
    const hasChanges = hasFieldChanges || hasNewEntries || hasDeletedEntries;

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
    $("#input-incremental").val("0.00");
    $("#input-redemption").val("0.00");
    updateMoneyHeaderLabels();
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
      const orderedMajorEntries =
        sortConfig.entrySortBy === "default"
          ? majorEntries
          : sortEntries(majorEntries);
      const minorRuns = buildMinorRuns(orderedMajorEntries);

      // Calculate total rows for a major (data rows + minor subtotal rows)
      let majorRowSpan = orderedMajorEntries.length;
      minorRuns.forEach((run) => {
        if (run.minorKey !== "__none__" && run.entries.length > 1) {
          majorRowSpan += 1; // add minor subtotal row
        }
      });

      let isFirstInMajor = true;

      minorRuns.forEach((run) => {
        const hasMinorSubtotal =
          run.minorKey !== "__none__" && run.entries.length > 1;
        const minorRowSpan = run.entries.length;
        let isFirstInMinor = true;

        run.entries.forEach((entry) => {
          const calc = calculated.entries[entry.id];
          const row = createEntryRow(
            entry,
            calc,
            isFirstInMajor,
            majorRowSpan,
            isFirstInMinor,
            minorRowSpan,
            run.minorCategory,
          );
          tbody.append(row);
          isFirstInMajor = false;
          isFirstInMinor = false;
        });

        // Minor category subtotal (only if more than 1 entry in this contiguous run)
        if (hasMinorSubtotal) {
          const minorSubtotal = calculateMinorRunSubtotal(
            run.entries,
            calculated.total.amount,
            calculated.entries,
          );
          tbody.append(createMinorSubtotalRow(run.minorCategory, minorSubtotal));
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
    restorePendingCellEditFocus();
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
                    <button class="btn btn-sm btn-danger btn-delete-entry" data-id="${entry.id}">
                        <i class="bi bi-trash"></i>
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
    $(".editable")
      .off("click")
      .on("click", function () {
        activateCellEditor($(this));
      });

    // Delete entry button
    $(".btn-delete-entry")
      .off("click")
      .on("click", function () {
        const id = $(this).data("id");
        deleteEntry(id);
      });
  }

  function activateCellEditor(cell) {
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
      currentValue = Number(entry.amount || 0).toFixed(2);
    } else {
      currentValue = entry[field] || "";
    }

    const inputType = field === "target_ratio" ? "number" : "text";
    const step = field === "target_ratio" ? "0.01" : "";
    const extraAttrs =
      field === "amount" ? 'inputmode="decimal" autocomplete="off"' : "";
    const input = $(
      `<input type="${inputType}" ${step ? `step="${step}"` : ""} ${extraAttrs} value="${currentValue}">`,
    );
    cell.html(input);
    input.focus().select();

    input.on("blur", function () {
      let newValue = $(this).val();

      if (field === "target_ratio") {
        newValue = parseFloat(newValue) / 100 || 0;
      } else if (field === "amount") {
        newValue = parseMoneyInput(newValue);
      } else {
        newValue = newValue.trim();
      }

      updateEntryField(id, field, newValue);

      // 小类编辑完成后，重新渲染表格以合并同类项
      if (field === "minor_category") {
        renderTable();
      }
    });

    input.on("keydown", function (e) {
      if (e.key === "Enter") {
        e.preventDefault();
        const nextRow = e.shiftKey
          ? row.prevAll("tr[data-id]").first()
          : row.nextAll("tr[data-id]").first();
        if (nextRow.length && nextRow.data("id") !== undefined) {
          setPendingCellEditFocus(nextRow.data("id"), field);
        } else {
          pendingCellEditFocus = null;
        }
        $(this).blur();
        return;
      }

      if (e.key === "Tab") {
        e.preventDefault();
        const direction = e.shiftKey ? -1 : 1;
        if (!setPendingCellEditFocusByCellOrder(cell, direction)) {
          pendingCellEditFocus = null;
        }
        $(this).blur();
        return;
      }

      if (e.key === "ArrowUp" || e.key === "ArrowDown") {
        e.preventDefault();
        const targetRow =
          e.key === "ArrowUp"
            ? row.prevAll("tr[data-id]").first()
            : row.nextAll("tr[data-id]").first();
        if (targetRow.length && targetRow.data("id") !== undefined) {
          setPendingCellEditFocus(targetRow.data("id"), field);
        } else {
          pendingCellEditFocus = null;
        }
        $(this).blur();
      }
    });
  }

  function restorePendingCellEditFocus() {
    if (!pendingCellEditFocus) return;
    const target = pendingCellEditFocus;
    pendingCellEditFocus = null;

    const row = $(`#fund-table-body tr[data-id="${target.entryId}"]`).first();
    if (!row.length) return;

    let cell = row.find(`td.editable[data-field="${target.field}"]`).first();
    if (!cell.length) {
      cell = row.find("td.editable").first();
    }
    if (!cell.length) return;

    activateCellEditor(cell);
  }

  function setPendingCellEditFocus(entryId, field) {
    pendingCellEditFocus = {
      entryId: entryId,
      field: field,
    };
  }

  function setPendingCellEditFocusByCellOrder(cell, direction) {
    const editableCells = $("#fund-table-body tr[data-id] td.editable");
    if (!editableCells.length) return false;

    const currentEl = cell.get(0);
    const currentIndex = editableCells.toArray().indexOf(currentEl);
    if (currentIndex < 0) return false;

    const targetCell = editableCells.eq(currentIndex + direction);
    if (!targetCell.length) return false;

    const targetRow = targetCell.closest("tr[data-id]");
    const targetId = targetRow.data("id");
    const targetField = targetCell.data("field");
    if (targetId === undefined || !targetField) return false;

    setPendingCellEditFocus(targetId, targetField);
    return true;
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
    return calcCore.distributeCentsProportionalByWeight(items, totalCents);
  }

  function calculateAll() {
    const totalAmount = entries.reduce((sum, e) => sum + e.amount, 0);
    const incrementalAmount = parseMoneyInput($("#input-incremental").val());
    const redemptionAmount = parseMoneyInput($("#input-redemption").val());
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
    return calcCore.calculateAllocationByMajorCategory(
      entries,
      totalAmount,
      incrementalAmount,
      sortConfig.majorOrder,
    );
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
    return calcCore.calculateRedemptionByMajorCategory(
      entries,
      totalAmount,
      redemptionAmount,
      sortConfig.majorOrder,
    );
  }

  function recalculateAll() {
    // Allow recalculation even when no backend portfolio is loaded (visitor local mode)
    renderTable();
  }

  // ============== Utilities ==============

  function groupByMajorCategory(entries) {
    return calcCore.groupByMajorCategory(entries, sortConfig.majorOrder);
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

  function buildMinorRuns(majorEntries) {
    const runs = [];

    majorEntries.forEach((entry) => {
      const minorKey = entry.minor_category || "__none__";
      const lastRun = runs[runs.length - 1];

      if (lastRun && lastRun.minorKey === minorKey) {
        lastRun.entries.push(entry);
      } else {
        runs.push({
          minorKey: minorKey,
          minorCategory: entry.minor_category || "",
          entries: [entry],
        });
      }
    });

    return runs;
  }

  function calculateMinorRunSubtotal(minorEntries, totalAmount, calcEntries) {
    const subtotal = {
      targetRatio: 0,
      amount: 0,
      actualRatio: 0,
      deviation: 0,
      rebalance: 0,
      allocation: 0,
      redemption: 0,
    };

    minorEntries.forEach((entry) => {
      const calc = calcEntries[entry.id] || {};
      subtotal.targetRatio += entry.target_ratio || 0;
      subtotal.amount += entry.amount || 0;
      subtotal.rebalance += calc.rebalance || 0;
      subtotal.allocation += calc.allocation || 0;
      subtotal.redemption += calc.redemption || 0;
    });

    subtotal.actualRatio = totalAmount > 0 ? subtotal.amount / totalAmount : 0;
    subtotal.deviation = subtotal.targetRatio - subtotal.actualRatio;

    return subtotal;
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
    if (value === 0) return `${moneyUnit}0.00`;
    const prefix = value < 0 ? "-" : "";
    const formatted = Math.abs(value)
      .toFixed(2)
      .replace(/\B(?=(\d{3})+(?!\d))/g, ",");
    return `${prefix}${moneyUnit}${formatted}`;
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

  function formatMoneyInputValue(value) {
    const num = parseMoneyInput(value);
    if (!Number.isFinite(num)) return "0.00";
    return num.toFixed(2);
  }

  // Parse money-like input safely: supports thousand separators and currency symbols.
  function parseMoneyInput(value) {
    if (value === null || value === undefined) return 0;
    if (typeof value === "number") return Number.isFinite(value) ? value : 0;

    let str = String(value).trim();
    if (!str) return 0;
    str = str.replace(/[，,\s￥¥$€£]/g, "");

    const num = Number(str);
    return Number.isFinite(num) ? num : 0;
  }

  function initMoneyUnit() {
    loadMoneyUnitForCurrentPortfolio();
    $("#money-unit-select").val(moneyUnit);
    updateMoneyHeaderLabels();
    if ($("#info-total").length) {
      $("#info-total").text(formatCurrency(0));
    }
  }

  function setMoneyUnit(unit) {
    if (!SUPPORTED_MONEY_UNITS.includes(unit)) return;
    moneyUnit = unit;
    $("#money-unit-select").val(moneyUnit);
    updateMoneyHeaderLabels();
    if (currentPortfolio && currentPortfolio.portfolio) {
      $("#info-total").text(formatCurrency(currentPortfolio.portfolio.total_amount));
      renderTable();
    } else if ($("#info-total").length) {
      $("#info-total").text(formatCurrency(0));
    }
    saveMoneyUnitForCurrentPortfolio();
  }

  function getMoneyUnitStorageKeyForCurrentPortfolio() {
    const id =
      currentPortfolio &&
      currentPortfolio.portfolio &&
      currentPortfolio.portfolio.id;
    if (id !== null && id !== undefined && id !== "") {
      return `${MONEY_UNIT_STORAGE_PREFIX}${id}`;
    }
    return MONEY_UNIT_STORAGE_SHARED_KEY;
  }

  function loadMoneyUnitForCurrentPortfolio() {
    try {
      const key = getMoneyUnitStorageKeyForCurrentPortfolio();
      let stored = localStorage.getItem(key);

      // Backward compatibility for previous global key
      if (!stored) {
        stored = localStorage.getItem(MONEY_UNIT_STORAGE_LEGACY_KEY);
      }

      if (stored && SUPPORTED_MONEY_UNITS.includes(stored)) {
        moneyUnit = stored;
      } else {
        moneyUnit = DEFAULT_MONEY_UNIT;
      }
    } catch (e) {
      console.warn("读取金额单位设置失败", e);
      moneyUnit = DEFAULT_MONEY_UNIT;
    }
    $("#money-unit-select").val(moneyUnit);
    updateMoneyHeaderLabels();
  }

  function saveMoneyUnitForCurrentPortfolio() {
    try {
      const key = getMoneyUnitStorageKeyForCurrentPortfolio();
      localStorage.setItem(key, moneyUnit);
    } catch (e) {
      console.warn("保存金额单位设置失败", e);
    }
  }

  function updateMoneyHeaderLabels() {
    $(".money-header").each(function () {
      const label = $(this).data("label") || "";
      $(this).text(`${label}(${moneyUnit})`);
    });
  }
})();
