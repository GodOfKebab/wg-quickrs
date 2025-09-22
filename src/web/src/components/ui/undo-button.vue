<template>
  <div :class="alignmentClasses" class="inline-block float-right absolute z-20">
    <button
        :disabled="disabled"
        class="align-middle p-0.5 rounded bg-gray-100 hover:bg-gray-500 hover:text-white transition cursor-pointer"
        title="Undo Changes"
        @click="$emit('click');">
      <img alt="Undo" class="h-6" src="/icons/flowbite/undo.svg"/>
    </button>
  </div>
</template>

<script>
export default {
  name: "undo-button",
  props: {
    disabled: false,
    alignmentClasses: [],
  },
  emits: ['click'],
}
</script>

<style scoped>

</style>