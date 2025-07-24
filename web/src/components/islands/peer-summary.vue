<template>

  <div :class="[color_div]" class="my-2 mr-2 p-1 shadow-md border rounded">
    <div class="my-0.5 truncate flex items-center relative">
                                   <span class="text-gray-800 text-xs mr-1">
                                     <strong class="text-sm">Name:</strong>
                                   </span>
      <input
          v-model="peer_local.name"
          :class="[FIELD_COLOR_LOOKUP[is_changed_field.name]]"
          class="rounded p-1 border-1 border-gray-100 focus:border-gray-200 outline-none w-full text-xs text-gray-500 grow"
          placeholder="Name" type="text"/>
      <div v-if="is_changed_field.name"
           class="inline-block float-right absolute z-20 right-[3px] top-[-1px]">
        <button
            :disabled="!is_changed_field.name"
            class="align-middle p-0.5 rounded bg-gray-100 hover:bg-gray-500 hover:text-white transition"
            title="Undo Changes"
            @click="peer_local.name = peer.name">
          <img alt="Undo" class="h-4" src="../../icons/flowbite/undo.svg"/>
        </button>
      </div>
    </div>

    <div class="mb-0.5 truncate flex items-center relative">
                                   <span class="text-gray-800 text-xs mr-1">
                                     <strong class="text-sm">Address:</strong>
                                   </span>
      <input
          v-model="peer_local.address"
          :class="[FIELD_COLOR_LOOKUP[is_changed_field.address]]"
          :placeholder="`Address (e.g. ${next_available_address})`"
          class="rounded p-1 border-1 border-gray-100 focus:border-gray-200 outline-none w-full text-xs text-gray-500 grow"
          type="text"
          @change=""/> <!--TODO: update connection address on change  -->
      <div v-if="is_changed_field.address"
           class="inline-block float-right absolute z-20 right-[3px] top-[-1px]">
        <button
            :disabled="!is_changed_field.address"
            class="align-middle p-0.5 rounded bg-gray-100 hover:bg-gray-500 hover:text-white transition undo-button-itself"
            title="Undo Changes"
            @click="peer_local.address = peer.address">
          <img alt="Undo" class="h-4" src="../../icons/flowbite/undo.svg"/>
        </button>
      </div>
    </div>

    <div class="form-check truncate flex items-center relative">
      <label class="form-check-label flex items-center">
        <input
            :checked="peer_local.mobility === 'static'"
            class="form-check-input appearance-none h-4 w-4 border border-gray-300 rounded-sm bg-white checked:bg-blue-600 checked:border-blue-600 focus:outline-none transition duration-200 bg-no-repeat bg-center bg-contain float-left mr-1 cursor-pointer"
            type="checkbox"
            @change="peer_local.mobility = peer_local.mobility === 'static' ? 'roaming' : 'static';">
        <span class="text-gray-800 cursor-pointer text-xs mr-1">
          <strong class="text-sm">Static Endpoint:</strong>
        </span>
      </label>

      <input
          v-model="peer_local.endpoint"
          :class="[FIELD_COLOR_LOOKUP[is_changed_field.endpoint]]" :disabled="peer_local.mobility !== 'static'"
          class="rounded p-1 border-1 border-gray-100 focus:border-gray-200 outline-none w-full text-xs text-gray-500 grow disabled:bg-gray-100"
          placeholder="Endpoint (e.g. 1.2.3.4:51820 example.com:51820)"
          type="text"/>
      <div
          v-if="is_changed_field.mobility || is_changed_field.endpoint"
          class="inline-block float-right absolute z-20 right-[3px] top-[-1px]">
        <button
            :disabled="!(is_changed_field.mobility || is_changed_field.endpoint)"
            class="align-middle p-0.5 rounded bg-gray-100 hover:bg-gray-500 hover:text-white transition"
            title="Undo Changes"
            @click="peer_local.endpoint = peer.endpoint; peer_local.mobility = peer.mobility;">
          <img alt="Undo" class="h-4" src="../../icons/flowbite/undo.svg"/>
        </button>
      </div>
    </div>
  </div>

</template>

<script>


import WireGuardHelper from "../../js/wg-helper.js";

export default {
  name: "peer-summary",
  props: {
    peer: {
      type: Object,
      default: {},
    },
  },
  data() {
    return {
      peer_local: {name: null, address: null, mobility: null, endpoint: null},
      island_change_sum: {
        changed_fields: {},
        errors: {},
      },
      FIELD_COLOR_LOOKUP: {
        0: 'bg-white',
        1: 'enabled:bg-green-200',
        '-1': 'enabled:bg-red-200',
      },
      is_changed_field: {name: 0, address: 0, mobility: 0, endpoint: 0},
      color_div: 'bg-green-50',
    };
  },
  created() {
    this.peer_local.name = this.peer.name;
    this.peer_local.address = this.peer.address;
    this.peer_local.mobility = this.peer.mobility;
    this.peer_local.endpoint = this.peer.endpoint;
  },
  emits: ['updated-change-sum'],
  methods: {
    check_field_status(field_name) {
      if (this.peer_local[field_name] === this.peer[field_name]) return [0, ''];
      const ret = WireGuardHelper.checkField(field_name, this.peer_local[field_name]);
      if (!ret.status) return [-1, ret.msg];
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