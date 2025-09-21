<template>

  <div :class="[color_div]" class="my-2 py-2 pl-1 pr-3 shadow-md border rounded">
    <div class="grid grid-cols-3 gap-2 mb-0.5">


      <!-- Kind -->
      <div class="truncate col-span-1">
        <div class="flex relative">
          <div class="flex items-center ml-2">
            <span class="text-gray-800 cursor-pointer text-xs">
              <strong class="text-sm"> Kind: </strong>
            </span>
          </div>

          <input v-model="peer_local.kind" :class="[FIELD_COLOR_LOOKUP[is_changed_field.kind]]"
                 class="rounded p-1 border-1 border-gray-100 focus:border-gray-200 outline-none text-xs text-gray-500 disabled:bg-gray-100 inline-block ml-1 w-full"
                 list="kind-recs"
                 type="text"/>
          <datalist id="kind-recs">
            <!--            <option :value="defaultKindIcon.kind">-->
            <!--              {{ defaultKindIcon.kind }}-->
            <!--            </option>-->
            <option value="server">Server</option>
            <option value="desktop">Desktop</option>
            <option value="laptop">Laptop</option>
            <option value="tablet">Tablet</option>
            <option value="phone">Phone</option>
            <option value="IoT">IoT</option>
            <option value="other">Other</option>
          </datalist>
          <div v-if="is_changed_field.kind"
               class="inline-block float-right absolute z-20 right-[5px] top-[0px]">
            <button
                :disabled="!is_changed_field.kind"
                class="align-middle p-0.5 rounded bg-gray-100 hover:bg-gray-500 hover:text-white transition"
                title="Undo Changes"
                @click="peer_local.kind = JSON.parse(JSON.stringify(peer.kind))">
              <img alt="Undo" class="h-4" src="/icons/flowbite/undo.svg"/>
            </button>
          </div>
        </div>
      </div>


      <!-- Icon -->
      <div class="truncate col-span-2">
        <div class="flex relative">
          <label class="form-check-label flex items-center">
            <input
                v-model="peer_local.icon.enabled"
                class="h-4 w-4"
                type="checkbox">
            <span class="text-gray-800 cursor-pointer text-xs">
                <strong class="text-sm"> Icon: </strong>
              </span>
          </label>
          <input v-model="peer_local.icon.value" :class="[FIELD_COLOR_LOOKUP[is_changed_field.icon]]"
                 :disabled="!peer_local.icon.enabled"
                 class="rounded p-1 border-1 border-gray-100 focus:border-gray-200 outline-none text-xs text-gray-500 disabled:bg-gray-100 inline-block ml-1 w-full"
                 list="icon-recs"
                 placeholder="(e.g. data:image/png;base64,iVBOR...)"
                 type="text"/>
          <datalist id="icon-recs">
            <option :value="defaultKindIcon.icon.value">
              {{ defaultKindIcon.icon.value }}
            </option>
          </datalist>
          <div v-if="is_changed_field.icon"
               class="inline-block float-right absolute z-20 right-[5px] top-[0px]">
            <button
                :disabled="!is_changed_field.icon"
                class="align-middle p-0.5 rounded bg-gray-100 hover:bg-gray-500 hover:text-white transition"
                title="Undo Changes"
                @click="peer_local.icon = JSON.parse(JSON.stringify(peer.icon))">
              <img alt="Undo" class="h-4" src="/icons/flowbite/undo.svg"/>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import WireGuardHelper from "@/js/wg-helper.js";
import FastEqual from "fast-deep-equal";


export default {
  name: "peer-kind-icon-island",
  props: {
    peer: {
      type: Object,
      default: {},
    },
    defaultKindIcon: {
      type: Object,
      default: {
        kind: "",
        icon: {enabled: false, value: ""},
      },
    },
  },
  data() {
    return {
      peer_local: {kind: "", icon: {enabled: false, value: ""}},
      island_change_sum: {
        changed_fields: {},
        errors: {},
      },
      FIELD_COLOR_LOOKUP: {
        0: 'bg-white',
        1: 'enabled:bg-green-200',
        '-1': 'enabled:bg-red-200',
      },
      is_changed_field: {kind: 0, icon: 0},
      color_div: 'bg-green-50',
    };
  },
  created() {
    this.peer_local.kind = JSON.parse(JSON.stringify(this.peer.kind));
    this.peer_local.icon = JSON.parse(JSON.stringify(this.peer.icon));
  },
  emits: ['updated-change-sum'],
  methods: {
    _fast_equal(s1, s2) {
      return FastEqual(s1, s2);
    },
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