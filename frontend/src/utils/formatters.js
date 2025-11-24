export function formatCurrency(value) {
  return value?.toLocaleString('en-US', {
    style: 'currency',
    currency: 'USD',
    maximumFractionDigits: 2,
  }) ?? '$0.00';
}

export function formatNumber(value) {
  return value?.toLocaleString('en-US') ?? '0';
}