import { expect, test } from "@playwright/test";
import { collectProblems, installPreviewAgentStub } from "./previewAgentStub";

test("opens on the demo start screen in a plain browser", async ({ page }) => {
  const problems = collectProblems(page);
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Agely" })).toBeVisible();
  await expect(page.getByRole("heading", { level: 2, name: "Experience shouldn't retire." })).toBeVisible();
  await expect(page.getByText("All data is scripted. No agents, payments or calendars are connected.", { exact: false })).toBeVisible();
  await expect(page.getByText("Demo — scripted data")).toBeVisible();
  await expect(page.getByText(/^Web preview v/)).toBeVisible();
  expect(problems).toEqual([]);
});

test("renders the version supplied by an explicit browser preview fixture", async ({ page }) => {
  const problems = collectProblems(page);
  await installPreviewAgentStub(page);
  await page.goto("/");
  await expect(page.getByText("Web preview v0.0.0-e2e")).toBeVisible();
  expect(problems).toEqual([]);
});
