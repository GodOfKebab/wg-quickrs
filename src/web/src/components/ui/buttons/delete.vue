<template>
  <button :disabled="disabled"
          class="align-middle disabled:opacity-40 bg-red-100 hover:enabled:bg-red-600 text-red-600 hover:enabled:text-white transition p-1 rounded"
          :title="title"
          @click="$emit('click');">
    <!-- "trash-bin" @ https://flowbite.com/icons/ -->
    <svg :class="imageClasses" class="fill-none" aria-hidden="true" xmlns="http://www.w3.org/2000/svg"
         width="24" height="24" viewBox="0 0 24 24">
      <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
            d="M5 7h14m-9 3v8m4-8v8M10 3h4a1 1 0 0 1 1 1v3H9V4a1 1 0 0 1 1-1ZM6 7h12v13a1 1 0 0 1-1 1H7a1 1 0 0 1-1-1V7Z"/>
    </svg>
  </button>
</template>
<script>
export default {
  name: "undo-button",
  props: {
    disabled: false,
    title: "",
    imageClasses: "",
  },
  emits: ['click'],
}
</script>

<style scoped>

</style>