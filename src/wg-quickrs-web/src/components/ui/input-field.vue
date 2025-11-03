<template>
  <div class="my-0.5 truncate flex items-center relative ml-2">
    <!-- Label -->
    <checkbox v-if="valueField" :checked="modelValue.enabled" :label="`${label}:`" class="mr-1" size="5" :disabled="disabled"
              @click="emit_ev(!modelValue.enabled, modelValue[valueField])"></checkbox>
    <field v-else :field="`${label}:`" class="mr-1"></field>


    <!-- Input -->
    <input
        :disabled="disabled || (valueField ? !modelValue.enabled : false)"
        :class="[inputColor]"
        :list="`${label}-list`"
        :placeholder="placeholder"
        :value="valueField ? modelValue[valueField] : modelValue"
        class="rounded pl-1.5 pt-[2px] pb-[2px] my-0.5 focus:outline-none focus:ring-0 border-1 border-gray-200 focus:border-gray-400 outline-none w-full text-lg text-gray-500 grow disabled:bg-gray-100"
        type="text"
        @input="valueField ? emit_ev(modelValue.enabled, $event.target.value) : $emit('update:modelValue', $event.target.value)"/>

    <!-- Undo Button -->
    <undo-button v-if="!_fast_equal(modelValue, valuePrev) && !disabled"
                 :disabled="_fast_equal(modelValue, valuePrev)"
                 :alignment-classes="undoButtonAlignmentClasses"
                 image-classes="h-5"
                 @click="$emit('update:modelValue', valuePrev);">
    </undo-button>
  </div>

</template>

<script>
import FastEqual from "fast-deep-equal";
import UndoButton from "@/src/components/ui/buttons/undo.vue";
import Checkbox from "@/src/components/ui/checkbox.vue";
import Field from "@/src/components/ui/field.vue";

export default {
  name: "input-field",
  components: {Field, Checkbox, UndoButton},
  props: {
    modelValue: null,
    valuePrev: null,
    label: "",
    placeholder: "",
    inputColor: "",
    disabled: false,
    valueField: null,
    undoButtonAlignmentClasses: ""
  },
  emits: ['update:modelValue'],
  methods: {
    _fast_equal(s1, s2) {
      return FastEqual(s1, s2);
    },
    emit_ev(enabled, value) {
      this.$emit('update:modelValue', {enabled, [this.valueField]: value});
    }
  },
}
</script>

<style scoped>

</style>