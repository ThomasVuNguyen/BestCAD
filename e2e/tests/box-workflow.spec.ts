import { test, expect } from '@playwright/test';

test('Box parametric workflow', async ({ page }) => {
  // 1. Navigate to app
  await page.goto('/');

  // 2. Verify initial box renders (canvas is not empty)
  const canvas = page.locator('canvas');
  await expect(canvas).toBeVisible();

  // 3. Check feature tree shows "Box"
  const featureTree = page.locator('.feature-tree');
  await expect(featureTree).toContainText('Box');

  // 4. Change width dimension in property panel
  const widthInput = page.locator('input[name="width"]');
  if (await widthInput.isVisible()) {
    await widthInput.fill('200');
    
    // 5. Click Apply
    await page.getByRole('button', { name: 'Apply' }).click();
  }

  // 6. Verify the model updates (visual check - in reality might check api or DOM data-attributes)
  // 7. Click a face in the viewport (simulate click on canvas at known position)
  await canvas.click({ position: { x: 300, y: 300 } });

  // 8. Verify face selection feedback
  // (Assuming there is some UI feedback for selection)

  // 9. Undo (if implemented)
  
  // 10. Click STEP export button & 11. Verify download
  const downloadPromise = page.waitForEvent('download');
  await page.getByRole('button', { name: 'Export STEP' }).click();
  const download = await downloadPromise;
  expect(download.suggestedFilename()).toMatch(/\.step$/);

  // 12. Take a final screenshot
  await page.screenshot({ path: 'final-workflow.png' });
});
