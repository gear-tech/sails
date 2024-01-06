export function getClassName(name: string) {
  if (!name) {
    return 'Service';
  }
  return name
    .split(/[\s_]/)
    .map((value) => `${value[0].toUpperCase()}${value.slice(1).toLowerCase()}`)
    .join('');
}
