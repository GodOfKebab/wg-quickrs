<template>

  <div :class="[computed_colors.div]" class="my-2 p-1 shadow-md border rounded">
    <div class="text-gray-800 mb-0.5">
      Configure DNS and MTU:
    </div>
    <div class="grid grid-cols-2 gap-2 mb-0.5">
      <div v-for="field in ['dns', 'mtu']">
        <div class="truncate">
          <div
              :class="[dnsmtu[field].enabled !== dnsmtu_local[field].enabled || dnsmtu[field].value !== dnsmtu_local[field].value ? 'highlight-undo-box' : '']"
              class="flex">
            <label class="form-check-label flex items-center">
              <input
                  v-model="dnsmtu_local[field].enabled"
                  class="form-check-input appearance-none h-4 w-4 border border-gray-300 rounded-sm bg-white checked:bg-blue-600 checked:border-blue-600 focus:outline-none transition duration-200 bg-no-repeat bg-center bg-contain float-left mr-1 cursor-pointer"
                  type="checkbox">
              <span class="text-gray-800 cursor-pointer text-xs">
                <strong class="text-sm">{{ field.toUpperCase() }}: </strong>
              </span>
            </label>
            <input v-model="dnsmtu_local[field].value" :class="[`${computed_colors[field]}`]"
                   :disabled="!dnsmtu_local[field].enabled"
                   :list="field + 'Recommendations'"
                   :placeholder="defaultDnsmtu[field].value !== '' ? 'Click to see recommendations' : 'No recommendations'"
                   class="rounded p-1 border-1 border-gray-100 focus:border-gray-200 outline-none text-xs text-gray-500 disabled:bg-gray-100 inline-block ml-1 w-full"
                   type="text"/>
            <datalist :id="field + 'Recommendations'">
              <option v-if="field === 'dns'" :value="defaultDnsmtu[field].value">
                Forward all DNS related traffic to {{ defaultDnsmtu[field].value }}
              </option>
              <option v-if="field === 'mtu'" :value="defaultDnsmtu[field].value">
                Set MTU to {{ defaultDnsmtu[field].value }}
              </option>
            </datalist>
            <div
                v-if="!(dnsmtu_local[field].enabled === dnsmtu[field].enabled && dnsmtu_local[field].value === dnsmtu[field].value)"
                class="inline-block float-right absolute z-20 right-[3px] top-[0px]">
              <button
                  :disabled="dnsmtu_local[field].enabled === dnsmtu[field].enabled && dnsmtu_local[field].value === dnsmtu[field].value"
                  class="align-middle p-0.5 rounded bg-gray-100 hover:bg-gray-500 hover:text-white transition"
                  title="Undo Changes"
                  @click="dnsmtu_local[field].enabled = dnsmtu[field].enabled; dnsmtu_local[field].value = dnsmtu[field].value;">
                <img alt="Undo" class="h-4" src="../../icons/flowbite/undo.svg"/>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import WireGuardHelper from "../../js/wg-helper.js";


export default {
  name: "dnsmtu-island",
  props: {
    dnsmtu: {
      type: Object,
      default: {
        dns: {enabled: false, value: ""},
        mtu: {enabled: false, value: ""},
      },
    },
    defaultDnsmtu: {
      type: Object,
      default: {
        dns: {enabled: false, value: ""},
        mtu: {enabled: false, value: ""},
      },
    },
    value: {
      type: Object,
      default: {
        context: 'edit',
        changedFields: {},
        error: null,
      },
    },
  },
  data() {
    return {
      dnsmtu_local: {},
    };
  },
  created() {
    this.dnsmtu_local = this.dnsmtu;
  },
  emits: ['update:value'],
  computed: {
    computed_colors() {
      const changedFields = {dns: {}, mtu: {}};
      let error = null;
      const colors = {};
      for (const field of ['dns', 'mtu']) {
        colors[field] = 'bg-white';
        if (this.value.context === 'create' ||
            this.dnsmtu_local[field].enabled !== this.dnsmtu[field].enabled ||
            this.dnsmtu_local[field].value !== this.dnsmtu[field].value) {
          colors[field] = WireGuardHelper.checkField(field, this.dnsmtu_local[field]) ? 'bg-green-200' : 'bg-red-200';
        }

        if (this.dnsmtu_local[field].enabled !== this.dnsmtu[field].enabled) changedFields[field].enabled = this.dnsmtu_local[field].enabled;
        if (this.dnsmtu_local[field].value !== this.dnsmtu[field].value) changedFields[field].value = this.dnsmtu_local[field].value;
        if (Object.keys(changedFields[field]).length === 0) delete changedFields[field];

        error = this.dnsmtu[field].enabled && colors[field] === 'bg-red-200' ? field : error;
      }

      colors.div = 'bg-gray-100';
      if (this.dnsmtu.dns.enabled || this.dnsmtu.mtu.enabled) {
        colors.div = 'bg-green-100';
        if ((this.dnsmtu.dns.enabled && colors.dns === 'bg-red-200') ||
            (this.dnsmtu.mtu.enabled && colors.mtu === 'bg-red-200')) {
          colors.div = 'bg-red-50';
        }
      }
      this.value.changedFields = changedFields;
      this.value.error = error;

      return colors;
    },
  },
}
</script>

<style scoped>

</style>