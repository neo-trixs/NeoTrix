import { test, expect } from '@playwright/test';
import { mockCommand } from './fixtures';

test('empty state + composer card check', async ({ page }) => {
  await mockCommand(page, "neocodex_list_sessions", () => []);
  await mockCommand(page, "neocodex_list_archived", () => []);
  await page.goto("/");
  await page.getByTestId("sidebar-tab-sessions").click();

  await expect(page.getByText("我们该做什么？")).toBeVisible({ timeout: 10_000 });

  const titleSize = await page.getByText("我们该做什么？").evaluate((el) => {
    const cs = getComputedStyle(el);
    return { fs: cs.fontSize, weight: cs.fontWeight, grad: cs.backgroundImage.includes("linear-gradient") };
  });
  console.log("TITLE:", JSON.stringify(titleSize));

  const card = page.locator("form > div").first();
  await expect(card).toBeVisible();
  const cardStyle = await card.evaluate((el) => {
    const cs = getComputedStyle(el);
    return { radius: cs.borderRadius, bg: cs.backgroundColor, maxW: cs.maxWidth };
  });
  console.log("CARD:", JSON.stringify(cardStyle));

  await expect(page.getByTestId("composer-permission")).toBeVisible();
  await expect(page.getByTestId("composer-permission")).toContainText("自动");

  await expect(page.getByTestId("composer-context")).toBeVisible();
  console.log("EMPTY-STATE OK");
});
