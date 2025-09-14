<template>

  <div :class="[color_div]" class="my-2 py-2 px-3 shadow-md border rounded">
    <div class="text-gray-800 mb-1">
      Configure DNS and MTU:
    </div>
    <div class="grid grid-cols-2 gap-2 mb-0.5">
      <div v-for="field in ['dns', 'mtu']">
        <div class="truncate">
          <div class="flex relative">
            <label class="form-check-label flex items-center">
              <input
                  v-model="peer_local[field].enabled"
                  class="form-check-input appearance-none h-4 w-4 border border-gray-300 rounded-sm bg-white checked:bg-blue-600 checked:border-blue-600 focus:outline-none transition duration-200 bg-no-repeat bg-center bg-contain float-left mr-1 cursor-pointer"
                  type="checkbox">
              <span class="text-gray-800 cursor-pointer text-xs">
                <strong class="text-sm"> {{ field.toLocaleUpperCase() }} </strong>
              </span>
            </label>
            <input v-model="peer_local[field].value" :class="[FIELD_COLOR_LOOKUP[is_changed_field[field]]]"
                   :disabled="!peer_local[field].enabled"
                   :list="`${field} Recommendations`"
                   :placeholder="defaultDnsmtu[field].value !== '' ? 'Click to see recommendations' : 'No recommendations'"
                   class="rounded p-1 border-1 border-gray-100 focus:border-gray-200 outline-none text-xs text-gray-500 disabled:bg-gray-100 inline-block ml-1 w-full"
                   type="text"/>
            <datalist :id="`${field} Recommendations`">
              <option :value="defaultDnsmtu[field].value">
                Forward all DNS related traffic to {{ defaultDnsmtu[field].value }}
              </option>
            </datalist>
            <div v-if="is_changed_field[field]"
                 class="inline-block float-right absolute z-20 right-[3px] top-[-1px]">
              <button
                  :disabled="!is_changed_field[field]"
                  class="align-middle p-0.5 rounded bg-gray-100 hover:bg-gray-500 hover:text-white transition"
                  title="Undo Changes"
                  @click="peer_local[field] = JSON.parse(JSON.stringify(peer[field]))">
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
import FastEqual from "fast-deep-equal";


export default {
  name: "dnsmtu-island",
  props: {
    peer: {
      type: Object,
      default: {},
    },
    defaultDnsmtu: {
      type: Object,
      default: {
        dns: {enabled: false, value: ""},
        mtu: {enabled: false, value: ""},
      },
    },
  },
  data() {
    return {
      peer_local: {dns: {enabled: false, value: ""}, mtu: {enabled: false, value: ""}},
      island_change_sum: {
        changed_fields: {},
        errors: {},
      },
      FIELD_COLOR_LOOKUP: {
        0: 'bg-white',
        1: 'enabled:bg-green-200',
        '-1': 'enabled:bg-red-200',
      },
      is_changed_field: {dns: 0, mtu: 0},
      color_div: 'bg-green-50',
    };
  },
  created() {
    this.peer_local.dns = JSON.parse(JSON.stringify(this.peer.dns));
    this.peer_local.mtu = JSON.parse(JSON.stringify(this.peer.mtu));
  },
  emits: ['updated-change-sum'],
  methods: {
    check_field_status(field_name) {
      const ret = WireGuardHelper.checkField(field_name, this.peer_local[field_name]);
      if (!ret.status) return [-1, ret.msg];
      if (FastEqual(this.peer_local[field_name], this.peer[field_name])) return [0, ''];
      return [1, ''];
    },
    emit_island_change_sum() {
      this.$emit("updated-change-sum", this.island_change_sum);
    }
  },
  watch: {
    peer_local: {
      handler() {
        let errorDetected = false;
        let changeDetected = false;
        for (let field in this.peer_local) {
          let msg = "";
          [this.is_changed_field[field], msg] = this.check_field_status(field);
          this.island_change_sum.errors[field] = this.is_changed_field[field] === -1 ? msg : null;
          this.island_change_sum.changed_fields[field] = this.is_changed_field[field] === 1 ? this.peer_local[field] : null;

          errorDetected ||= this.is_changed_field[field] === -1;
          changeDetected ||= this.is_changed_field[field] !== 0;
        }
        this.emit_island_change_sum();
        this.color_div = errorDetected ? 'bg-red-50' : changeDetected ? 'bg-green-100' : 'bg-green-50';
      },
      deep: true,
    }
  },
}
</script>

<style scoped>

</style>