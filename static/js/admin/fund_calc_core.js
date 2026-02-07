/* eslint-disable no-var */
(function (root, factory) {
  if (typeof module === "object" && module.exports) {
    module.exports = factory();
    return;
  }
  root.FundCalcCore = factory();
})(typeof globalThis !== "undefined" ? globalThis : this, function () {
  "use strict";

  function distributeCentsProportionalByWeight(items, totalCents) {
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

  function groupByMajorCategory(entries, majorOrder) {
    const groups = {};
    entries.forEach((entry) => {
      const cat = entry.major_category;
      if (!groups[cat]) groups[cat] = [];
      groups[cat].push(entry);
    });

    const orderedGroups = {};
    (majorOrder || []).forEach((cat) => {
      if (groups[cat]) orderedGroups[cat] = groups[cat];
    });
    Object.keys(groups).forEach((cat) => {
      if (!orderedGroups[cat]) orderedGroups[cat] = groups[cat];
    });
    return orderedGroups;
  }

  function calculateAllocationByMajorCategory(
    entries,
    totalAmount,
    incrementalAmount,
    majorOrder,
  ) {
    const result = {};
    entries.forEach((e) => (result[e.id] = 0));
    if (incrementalAmount <= 0 || totalAmount <= 0) return result;

    const newTotalAmount = totalAmount + incrementalAmount;
    const majorGroups = groupByMajorCategory(entries, majorOrder);
    const majorData = {};
    let totalNeedAllocate = 0;

    Object.keys(majorGroups).forEach((majorCat) => {
      const majorEntries = majorGroups[majorCat];
      const targetRatio = majorEntries.reduce((sum, e) => sum + e.target_ratio, 0);
      const currentAmount = majorEntries.reduce((sum, e) => sum + e.amount, 0);
      const targetAmountAfterAllocate = newTotalAmount * targetRatio;
      const needAllocate = targetAmountAfterAllocate - currentAmount;

      majorData[majorCat] = {
        targetRatio,
        currentAmount,
        needAllocate,
        entries: majorEntries,
      };
      if (needAllocate > 0) totalNeedAllocate += needAllocate;
    });

    if (totalNeedAllocate <= 0) return result;

    const resultCents = {};
    entries.forEach((e) => (resultCents[e.id] = 0));
    const incrementalCents = Math.round(incrementalAmount * 100);

    Object.keys(majorData).forEach((majorCat) => {
      const data = majorData[majorCat];
      if (data.needAllocate <= 0) return;

      const majorAllocation =
        incrementalAmount * (data.needAllocate / totalNeedAllocate);
      const majorAllocationCents = Math.round(majorAllocation * 100);
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

      const majorDistributed = distributeCentsProportionalByWeight(
        localFunds,
        majorAllocationCents,
      );
      Object.keys(majorDistributed).forEach((id) => {
        resultCents[id] = (resultCents[id] || 0) + majorDistributed[id];
      });
    });

    const allocatedTotalCents = Object.keys(resultCents).reduce(
      (s, k) => s + (resultCents[k] || 0),
      0,
    );
    const globalDiffCents = incrementalCents - allocatedTotalCents;
    if (globalDiffCents !== 0) {
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

    Object.keys(resultCents).forEach((k) => {
      result[k] = (resultCents[k] || 0) / 100;
    });
    return result;
  }

  function calculateRedemptionByMajorCategory(
    entries,
    totalAmount,
    redemptionAmount,
    majorOrder,
  ) {
    const result = {};
    const resultCents = {};
    entries.forEach((e) => {
      resultCents[e.id] = 0;
      result[e.id] = 0;
    });

    const newTotalAmount = totalAmount - redemptionAmount;
    if (newTotalAmount <= 0) {
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

    const majorGroups = groupByMajorCategory(entries, majorOrder);
    const majorData = {};
    let totalNeedRedeem = 0;

    Object.keys(majorGroups).forEach((majorCat) => {
      const majorEntries = majorGroups[majorCat];
      const targetRatio = majorEntries.reduce((sum, e) => sum + e.target_ratio, 0);
      const currentAmount = majorEntries.reduce((sum, e) => sum + e.amount, 0);
      const targetAmountAfterRedeem = newTotalAmount * targetRatio;
      const needRedeem = currentAmount - targetAmountAfterRedeem;

      majorData[majorCat] = {
        targetRatio,
        currentAmount,
        needRedeem,
        entries: majorEntries,
      };
      if (needRedeem > 0) totalNeedRedeem += needRedeem;
    });

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

    Object.keys(majorData).forEach((majorCat) => {
      const data = majorData[majorCat];
      if (data.needRedeem <= 0) return;

      const majorRedemption =
        redemptionAmount * (data.needRedeem / totalNeedRedeem);
      const majorRedemptionCents = Math.round(majorRedemption * 100);
      const majorAmountAfterRedeem = data.currentAmount - majorRedemption;
      const needList = [];

      data.entries.forEach((entry) => {
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

      const distributed =
        needList.length > 0
          ? distributeCentsProportionalByWeight(needList, majorRedemptionCents)
          : distributeCentsProportionalByWeight(
              data.entries
                .filter((e) => e.amount > 0)
                .map((e) => ({ id: e.id, weight: e.amount })),
              majorRedemptionCents,
            );

      Object.keys(distributed).forEach((id) => {
        resultCents[id] = (resultCents[id] || 0) + distributed[id];
      });
    });

    const redemptionTotalCents = Math.round(redemptionAmount * 100);
    const allocatedTotalCents = Object.keys(resultCents).reduce(
      (s, k) => s + (resultCents[k] || 0),
      0,
    );
    const globalDiff = redemptionTotalCents - allocatedTotalCents;
    if (globalDiff !== 0) {
      let targetId = null;
      let maxAmount = -Infinity;
      entries.forEach((e) => {
        if (e.amount > maxAmount) {
          maxAmount = e.amount;
          targetId = e.id;
        }
      });
      if (!targetId && entries.length > 0) targetId = entries[0].id;
      if (targetId) {
        resultCents[targetId] = (resultCents[targetId] || 0) + globalDiff;
      }
    }

    Object.keys(resultCents).forEach((k) => {
      result[k] = (resultCents[k] || 0) / 100;
    });
    return result;
  }

  return {
    distributeCentsProportionalByWeight,
    groupByMajorCategory,
    calculateAllocationByMajorCategory,
    calculateRedemptionByMajorCategory,
  };
});
