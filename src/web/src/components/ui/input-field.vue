<template>
  <div class="my-0.5 truncate flex items-center relative ml-2">

    <!-- Label -->
    <div v-if="isEnabledValue" class="inline-flex items-center">
      <label class="flex items-center cursor-pointer relative">
        <!-- Checkbox -->
        <input
            :checked="modelValue.enabled"
            class="h-6 w-6 cursor-pointer transition-all appearance-none rounded shadow hover:shadow-md border bg-gray-100 border-slate-300 checked:bg-blue-600 checked:border-blue-600"
            type="checkbox"
            @change="$emit('update:modelValue',{enabled: !modelValue.enabled, value: modelValue.value})"/>

        <!-- Checkmark Icon -->
        <span v-if="modelValue.enabled"
              class="absolute text-white opacity-100 top-1/2 left-1 transform -translate-y-1/2 pointer-events-none">
          <svg class="h-4.5 w-4.5" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg">
            <path clip-rule="evenodd"
                  d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                  fill="currentColor" fill-rule="evenodd"></path>
          </svg>
        </span>

        <!-- Label text -->
        <span class="ml-2 text-gray-800 flex items-center mr-1">
          <strong class="text-xl mt-[1px]">{{ fieldLabel }}:</strong>
        </span>
      </label>
    </div>
    <span v-else class="text-gray-800 flex items-center mr-1">
      <strong class="text-xl mt-[1px]">{{ fieldLabel }}:</strong>
    </span>

    <!-- Input -->
    <input
        :class="[inputColor]"
        :disabled="isNewPeer || (isEnabledValue ? !modelValue.enabled : false)"
        :placeholder="fieldPlaceholder"
        :value="isEnabledValue ? modelValue.value : modelValue"
        class="rounded pl-1.5 pt-[2px] pb-[1px] my-0.5 focus:outline-none focus:ring-0 border-1 border-gray-200 focus:border-gray-400 outline-none w-full text-lg text-gray-500 grow disabled:bg-gray-100"
        type="text"
        @input="$emit('update:modelValue', isEnabledValue ? {enabled: modelValue.enabled, value: $event.target.value} : $event.target.value)"/>

    <!-- Undo Button -->
    <div v-if="!_fast_equal(modelValue, valuePrev) && !isNewPeer"
         class="inline-block float-right absolute z-20 right-[5px] top-[5px]">
      <button
          :disabled="_fast_equal(modelValue, valuePrev)"
          class="align-middle p-[1px] rounded bg-gray-100 hover:bg-gray-500 hover:text-white transition cursor-pointer"
          title="Undo Changes"
          @click="$emit('update:modelValue', valuePrev);">
        <img alt="Undo" class="h-6" src="/icons/flowbite/undo.svg"/>
      </button>
    </div>
  </div>

</template>

<script>
import FastEqual from "fast-deep-equal";

export default {
  name: "input-field",
  props: {
    modelValue: null,
    valuePrev: null,
    fieldLabel: "",
    fieldPlaceholder: "",
    inputColor: "",
    isNewPeer: false,
    isEnabledValue: false
  },
  emits: ['update:modelValue'],
  methods: {
    _fast_equal(s1, s2) {
      return FastEqual(s1, s2);
    },
  },
}
</script>

<style scoped>

</style>