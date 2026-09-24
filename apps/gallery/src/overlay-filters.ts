/** Filter functions supported by the native backdrop-filter parser. */
export const overlayFilters = [
  { name: 'Blur', value: 'blur(8px)' },
  { name: 'Brightness', value: 'brightness(1.5)' },
  { name: 'Contrast', value: 'contrast(1.7)' },
  { name: 'Saturate', value: 'saturate(2)' },
  { name: 'Opacity', value: 'opacity(55%)' },
  { name: 'Hue rotate', value: 'hue-rotate(110deg)' },
  { name: 'Grayscale', value: 'grayscale(90%)' },
  { name: 'Invert', value: 'invert(75%)' },
  { name: 'Sepia', value: 'sepia(90%)' },
  { name: 'Drop shadow', value: 'drop-shadow(4px 6px 8px #000000aa)' },
  { name: 'Refraction', value: 'refraction(8px 2px 0.2)' },
  { name: 'Color matrix', value: 'color-matrix(0.5 0 0 0 0 0 1 0 0 0 0 0 1.5 0 0 0 0 0 1 0)' },
] as const
