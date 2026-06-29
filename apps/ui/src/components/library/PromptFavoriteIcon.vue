<template>
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="16"
    height="16"
    viewBox="0 0 16 16"
    fill="currentColor"
    aria-hidden="true"
    class="lib-icon favorite"
  >
    <path
      d="M3 2.5A1.5 1.5 0 0 1 4.5 1h5A1.5 1.5 0 0 1 11 2.5V3h1.5A1.5 1.5 0 0 1 14 4.5v9a1.5 1.5 0 0 1-1.5 1.5h-9A1.5 1.5 0 0 1 2 13.5v-9A1.5 1.5 0 0 1 3.5 3H3V2.5Zm1.5-.5a.5.5 0 0 0-.5.5V3h6V2.5a.5.5 0 0 0-.5-.5h-5ZM3.5 4a.5.5 0 0 0-.5.5v8a.5.5 0 0 0 .5.5h9a.5.5 0 0 0 .5-.5v-8a.5.5 0 0 0-.5-.5h-9Z"
    />
    <path
      d="M8 5.5 9.1 7.9l2.6.38-1.88 1.83.44 2.58L8 11.4l-2.26 1.19.44-2.58L4.3 8.28l2.6-.38L8 5.5Z"
      class="star"
    />
  </svg>
</template>

<style scoped>
.lib-icon {
  flex-shrink: 0;
  color: var(--color-accent);
}

.star {
  fill: var(--color-warning);
}
</style>
