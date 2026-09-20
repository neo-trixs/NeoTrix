import { test, expect } from "./fixtures";
test("diag stability", async ({ page }) => {
  await page.goto("/");
  await page.waitForTimeout(1500);
  const info = await page.evaluate(() => {
    const seg = document.querySelector('.segb[data-view="cowork"]');
    const style = getComputedStyle(seg);
    return {
      segAnim: style.animationName,
      segRect: seg.getBoundingClientRect().toJSON(),
      htmlAnim: getComputedStyle(document.documentElement).animationName,
      userbarAnim: getComputedStyle(document.querySelector('#userBar')).animationName,
      styleSheets: document.styleSheets.length,
      bodyChildren: document.body.children.length,
    };
  });
  console.log("DIAG", JSON.stringify(info));
  const r1 = await page.locator('.segb[data-view="cowork"]').boundingBox();
  await page.waitForTimeout(700);
  const r2 = await page.locator('.segb[data-view="cowork"]').boundingBox();
  console.log("DIAG r1", JSON.stringify(r1), "r2", JSON.stringify(r2));
  expect(r1.x).toBe(r2.x);
});
