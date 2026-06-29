<template>
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="16"
    height="16"
    viewBox="0 0 16 16"
    fill="currentColor"
    aria-hidden="true"
    class="lib-icon"
  >
    <path
      d="M3 2.5A1.5 1.5 0 0 1 4.5 1h5A1.5 1.5 0 0 1 11 2.5V3h1.5A1.5 1.5 0 0 1 14 4.5v9a1.5 1.5 0 0 1-1.5 1.5h-9A1.5 1.5 0 0 1 2 13.5v-9A1.5 1.5 0 0 1 3.5 3H3V2.5Zm1.5-.5a.5.5 0 0 0-.5.5V3h6V2.5a.5.5 0 0 0-.5-.5h-5ZM3.5 4a.5.5 0 0 0-.5.5v8a.5.5 0 0 0 .5.5h9a.5.5 0 0 0 .5-.5v-8a.5.5 0 0 0-.5-.5h-9Z"
    />
    <circle cx="12.25" cy="12.25" r="3.25" class="badge" />
    <path
      d="M12.25 10.6v3.3M10.6 12.25h3.3"
      class="plus"
      fill="none"
      stroke="currentColor"
      stroke-width="1.1"
      stroke-linecap="round"
    />
  </svg>
</template>

<style scoped>
.lib-icon {
  flex-shrink: 0;
}

.badge {
  fill: var(--color-accent-muted);
  stroke: var(--color-accent);
  stroke-width: 0.75;
}

.plus {
  color: var(--color-accent);
}
</style>
