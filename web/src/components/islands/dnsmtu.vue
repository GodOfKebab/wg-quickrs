<template>

  <div :class="[color_div]" class="my-2 p-1 shadow-md border rounded">
    <div class="text-gray-800 mb-0.5">
      Configure DNS and MTU:
    </div>
    <div class="grid grid-cols-2 gap-2 mb-0.5">

      <!-- DNS -->
      <div class="truncate">
        <div class="flex relative">
          <label class="form-check-label flex items-center">
            <input
                v-model="peer_local.dns.enabled"
                class="form-check-input appearance-none h-4 w-4 border border-gray-300 rounded-sm bg-white checked:bg-blue-600 checked:border-blue-600 focus:outline-none transition duration-200 bg-no-repeat bg-center bg-contain float-left mr-1 cursor-pointer"
                type="checkbox">
            <span class="text-gray-800 cursor-pointer text-xs">
                <strong class="text-sm">DNS: </strong>
              </span>
          </label>
          <input v-model="peer_local.dns.value" :class="[FIELD_COLOR_LOOKUP[is_changed_dns]]"
                 :disabled="!peer_local.dns.enabled"
                 :list="'DNS Recommendations'"
                 :placeholder="defaultDnsmtu.dns.value !== '' ? 'Click to see recommendations' : 'No recommendations'"
                 class="rounded p-1 border-1 border-gray-100 focus:border-gray-200 outline-none text-xs text-gray-500 disabled:bg-gray-100 inline-block ml-1 w-full"
                 type="text"/>
          <datalist :id="'DNS Recommendations'">
            <option :value="defaultDnsmtu.dns.value">
              Forward all DNS related traffic to {{ defaultDnsmtu.dns.value }}
            </option>
          </datalist>
          <div v-if="is_changed_dns"
               class="inline-block float-right absolute z-20 right-[3px] top-[-1px]">
            <button
                :disabled="!is_changed_dns"
                class="align-middle p-0.5 rounded bg-gray-100 hover:bg-gray-500 hover:text-white transition"
                title="Undo Changes"
                @click="peer_local.dns = JSON.parse(JSON.stringify(peer.dns))">
              <img alt="Undo" class="h-4" src="../../icons/flowbite/undo.svg"/>
            </button>
          </div>
        </div>
      </div>

      <!-- MTU -->
      <div class="truncate">
        <div class="flex relative">
          <label class="form-check-label flex items-center">
            <input
                v-model="peer_local.mtu.enabled"
                class="form-check-input appearance-none h-4 w-4 border border-gray-300 rounded-sm bg-white checked:bg-blue-600 checked:border-blue-600 focus:outline-none transition duration-200 bg-no-repeat bg-center bg-contain float-left mr-1 cursor-pointer"
                type="checkbox">
            <span class="text-gray-800 cursor-pointer text-xs">
                <strong class="text-sm">MTU: </strong>
              </span>
          </label>
          <input v-model="peer_local.mtu.value" :class="[FIELD_COLOR_LOOKUP[is_changed_mtu]]"
                 :disabled="!peer_local.mtu.enabled"
                 :list="'DNS Recommendations'"
                 :placeholder="defaultDnsmtu.mtu.value !== '' ? 'Click to see recommendations' : 'No recommendations'"
                 class="rounded p-1 border-1 border-gray-100 focus:border-gray-200 outline-none text-xs text-gray-500 disabled:bg-gray-100 inline-block ml-1 w-full"
                 type="text"/>
          <datalist :id="'DNS Recommendations'">
            <option :value="defaultDnsmtu.mtu.value">
              Forward all DNS related traffic to {{ defaultDnsmtu.mtu.value }}
            </option>
          </datalist>
          <div v-if="is_changed_mtu"
               class="inline-block float-right absolute z-20 right-[3px] top-[-1px]">
            <button
                :disabled="!is_changed_mtu"
                class="align-middle p-0.5 rounded bg-gray-100 hover:bg-gray-500 hover:text-white transition"
                title="Undo Changes"
                @click="peer_local.mtu = JSON.parse(JSON.stringify(peer.mtu))">
              <img alt="Undo" class="h-4" src="../../icons/flowbite/undo.svg"/>
            </button>
          </div>
        </div>
      </div>


    </div>
  </div>
</template>

<script>
import FastEqual from "fast-deep-equal";

import WireGuardHelper from "../../js/wg-helper.js";


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
        changed_fields: {dns: {enabled: false, value: ""}, mtu: {enabled: false, value: ""}},
        errors: {dns: {enabled: false, value: ""}, mtu: {enabled: false, value: ""}},
      },
      FIELD_COLOR_LOOKUP: {
        0: 'bg-white',
        1: 'enabled:bg-green-200',
        '-1': 'enabled:bg-red-200',
      },
    };
  },
  created() {
    this.peer_local.dns = JSON.parse(JSON.stringify(this.peer.dns));
    this.peer_local.mtu = JSON.parse(JSON.stringify(this.peer.mtu));
  },
  emits: ['updated-change-sum'],
  methods: {
    check_field_status(field_name) {
      console.log(this.peer_local[field_name], this.peer[field_name])
      if (FastEqual(this.peer_local[field_name], this.peer[field_name])) return 0;
      if (!WireGuardHelper.checkField(field_name, this.peer_local[field_name])) return -1;
      return 1;
    },
    emit_island_change_sum() {
      this.$emit("updated-change-sum", this.island_change_sum);
    }
  },
  computed: {
    is_changed_dns() {
      const field_status = this.check_field_status('dns');
      this.island_change_sum.errors.dns = field_status === -1 ? 'DNS is invalid' : null;
      this.island_change_sum.changed_fields.dns = field_status === 1 ? this.peer_local.dns : null;
      this.emit_island_change_sum();
      return field_status;
    },
    is_changed_mtu() {
      const field_status = this.check_field_status('mtu');
      this.island_change_sum.errors.mtu = field_status === -1 ? 'MTU is invalid' : null;
      this.island_change_sum.changed_fields.mtu = field_status === 1 ? this.peer_local.mtu : null;
      this.emit_island_change_sum();
      return field_status;
    },
    color_div() {
      let changeDetected = false;
      for (const field_status of [this.is_changed_dns, this.is_changed_mtu]) {
        if (field_status === -1) {
          return 'bg-red-50';
        }
        if (field_status === 1) {
          changeDetected = true;
        }
      }
      return changeDetected ? 'bg-green-100' : 'bg-green-50';
    },
  },
}
</script>

<style scoped>

</style>