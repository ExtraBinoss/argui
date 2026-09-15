// Let browser reload commands bypass the canvas event handlers without cancelling
// their native default action. Ordinary widget keyboard input is unaffected.
window.addEventListener("keydown", (event) => {
  const target = event.target;
  if (!(target instanceof Element) ||
      !target.closest("canvas, [data-argui-accessibility]")) return;
  const reload = event.key === "F5" ||
    ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "r");
  if (reload && !event.altKey) event.stopPropagation();
}, { capture: true });
