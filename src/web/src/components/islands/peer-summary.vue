<template>

  <div :class="[color_div]" class="my-2 py-2 pl-1 pr-3 shadow-md border rounded">

    <!--  Name  -->
    <div class="my-0.5 truncate flex items-center relative ml-2">
                                   <span class="text-gray-800 text-xs mr-1">
                                     <strong class="text-sm">Name:</strong>
                                   </span>
      <input
          v-model="peer_local.name"
          :class="[FIELD_COLOR_LOOKUP[is_changed_field.name]]"
          class="rounded p-1 border-1 border-gray-100 focus:border-gray-200 outline-none w-full text-xs text-gray-500 grow"
          placeholder="Name" type="text"/>
      <div v-if="!_fast_equal(peer_local.name, peer.name)"
           class="inline-block float-right absolute z-20 right-[5px] top-[3px]">
        <button
            :disabled="_fast_equal(peer_local.name, peer.name)"
            class="align-middle p-0.5 rounded bg-gray-100 hover:bg-gray-500 hover:text-white transition"
            title="Undo Changes"
            @click="peer_local.name = peer.name">
          <img alt="Undo" class="h-4" src="/icons/flowbite/undo.svg"/>
        </button>
      </div>
    </div>

    <!--  Address  -->
    <div class="mb-0.5 truncate flex items-center relative ml-2">
                                   <span class="text-gray-800 text-xs mr-1">
                                     <strong class="text-sm">Address:</strong>
                                   </span>
      <input
          v-model="peer_local.address"
          :class="[FIELD_COLOR_LOOKUP[is_changed_field.address]]"
          :placeholder="`Address (e.g. 10.8.0.1})`"
          :disabled="isNewPeer"
          class="rounded p-1 border-1 border-gray-100 focus:border-gray-200 outline-none w-full text-xs text-gray-500 grow disabled:bg-gray-100"
          type="text"
          @change=""/> <!--TODO: update connection address on change  -->
      <!--TODO: get placeholder address to have an actual working address  -->
      <div v-if="!_fast_equal(peer_local.address, peer.address) && !isNewPeer"
           class="inline-block float-right absolute z-20 right-[5px] top-[3px]">
        <button
            :disabled="_fast_equal(peer_local.address, peer.address)"
            class="align-middle p-0.5 rounded bg-gray-100 hover:bg-gray-500 hover:text-white transition undo-button-itself"
            title="Undo Changes"
            @click="peer_local.address = peer.address">
          <img alt="Undo" class="h-4" src="/icons/flowbite/undo.svg"/>
        </button>
      </div>
    </div>

    <!--  Endpoint  -->
    <div class="form-check truncate flex items-center relative">
      <label class="form-check-label flex items-center">
        <input
            :checked="peer_local.endpoint.enabled"
            class="h-4 w-4"
            type="checkbox"
            :disabled="isHost"
            @change="peer_local.endpoint.enabled = !peer_local.endpoint.enabled;">
        <span class="text-gray-800 cursor-pointer text-xs mr-1">
          <strong class="text-sm">Static Endpoint:</strong>
        </span>
      </label>

      <input
          v-model="peer_local.endpoint.value"
          :class="[FIELD_COLOR_LOOKUP[is_changed_field.endpoint.value]]"
          :disabled="!peer_local.endpoint.enabled || isHost"
          class="rounded p-1 border-1 border-gray-100 focus:border-gray-200 outline-none w-full text-xs text-gray-500 grow disabled:bg-gray-100"
          placeholder="Endpoint (e.g. 1.2.3.4:51820 example.com:51820)"
          type="text"/>
      <div
          v-if="!_fast_equal(peer_local.endpoint, peer.endpoint)"
          class="inline-block float-right absolute z-20 right-[5px] top-[3px]">
        <button
            :disabled="_fast_equal(peer_local.endpoint, peer.endpoint)"
            class="align-middle p-0.5 rounded bg-gray-100 hover:bg-gray-500 hover:text-white transition"
            title="Undo Changes"
            @click="peer_local.endpoint.value = peer.endpoint.value; peer_local.endpoint.enabled = peer.endpoint.enabled;">
          <img alt="Undo" class="h-4" src="/icons/flowbite/undo.svg"/>
        </button>
      </div>
    </div>
  </div>

</template>

<script>


import WireGuardHelper from "@/js/wg-helper.js";
import FastEqual from "fast-deep-equal";

export default {
  name: "peer-summary",
  props: {
    peer: {
      type: Object,
      default: {},
    },
    isHost: {
      type: Boolean,
      default: false,
    },
    isNewPeer: {
      type: Boolean,
      default: false,
    },
  },
  data() {
    return {
      peer_local: {name: null, address: null, endpoint: {enabled: null, value: null}},
      island_change_sum: {
        changed_fields: {},
        errors: {},
      },
      FIELD_COLOR_LOOKUP: {
        0: 'bg-white',
        1: 'enabled:bg-green-200',
        '-1': 'enabled:bg-red-200',
      },
      is_changed_field: {name: 0, address: 0, endpoint: {enabled: 0, value: 0}},
      color_div: 'bg-green-50',
    };
  },

  created() {
    this.peer_local.name = JSON.parse(JSON.stringify(this.peer.name));
    this.peer_local.address = JSON.parse(JSON.stringify(this.peer.address));
    this.peer_local.endpoint = JSON.parse(JSON.stringify(this.peer.endpoint));
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