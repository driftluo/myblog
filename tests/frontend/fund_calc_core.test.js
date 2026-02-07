const test = require("node:test");
const assert = require("node:assert/strict");

const {
  distributeCentsProportionalByWeight,
  calculateAllocationByMajorCategory,
  calculateRedemptionByMajorCategory,
} = require("../../static/js/admin/fund_calc_core.js");

const MAJOR_ORDER = ["股票", "债券", "大宗商品", "现金"];

function sumAmountMapToCents(map) {
  return Object.values(map).reduce((sum, amount) => sum + Math.round(amount * 100), 0);
}

test("distributeCentsProportionalByWeight keeps total cents", () => {
  const result = distributeCentsProportionalByWeight(
    [
      { id: 1, weight: 3 },
      { id: 2, weight: 2 },
      { id: 3, weight: 1 },
    ],
    1001,
  );
  const total = Object.values(result).reduce((sum, cents) => sum + cents, 0);
  assert.equal(total, 1001);
});

test("allocation keeps total incremental amount", () => {
  const entries = [
    { id: 1, major_category: "股票", target_ratio: 0.6, amount: 600 },
    { id: 2, major_category: "债券", target_ratio: 0.4, amount: 400 },
  ];
  const result = calculateAllocationByMajorCategory(entries, 1000, 123.45, MAJOR_ORDER);
  assert.equal(sumAmountMapToCents(result), 12345);
  Object.values(result).forEach((v) => assert.ok(v >= 0));
});

test("redemption keeps total redemption amount", () => {
  const entries = [
    { id: 1, major_category: "股票", target_ratio: 0.7, amount: 700 },
    { id: 2, major_category: "债券", target_ratio: 0.3, amount: 300 },
  ];
  const result = calculateRedemptionByMajorCategory(entries, 1000, 234.56, MAJOR_ORDER);
  assert.equal(sumAmountMapToCents(result), 23456);
  Object.values(result).forEach((v) => assert.ok(v >= 0));
});

test("redemption larger than total redeems all holdings", () => {
  const entries = [
    { id: 1, major_category: "股票", target_ratio: 0.5, amount: 123.45 },
    { id: 2, major_category: "债券", target_ratio: 0.5, amount: 234.56 },
  ];
  const totalAmount = 358.01;
  const result = calculateRedemptionByMajorCategory(entries, totalAmount, 500, MAJOR_ORDER);
  assert.equal(sumAmountMapToCents(result), 35801);
});
