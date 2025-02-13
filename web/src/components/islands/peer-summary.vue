<template>

  <div :class="[color_div]" class="my-2 mr-2 p-1 shadow-md border rounded">
    <div class="my-0.5 truncate flex items-center relative">
                                   <span class="text-gray-800 text-xs mr-1">
                                     <strong class="text-sm">Name:</strong>
                                   </span>
      <input
          v-model="peer_local.name"
          :class="[FIELD_COLOR_LOOKUP[is_changed_name]]"
          class="rounded p-1 border-1 border-gray-100 focus:border-gray-200 outline-none w-full text-xs text-gray-500 grow"
          placeholder="Name" type="text"/>
      <div v-if="is_changed_name"
           class="inline-block float-right absolute z-20 right-[3px] top-[-1px]">
        <button
            :disabled="!is_changed_name"
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
          :class="[FIELD_COLOR_LOOKUP[is_changed_address]]"
          :placeholder="`Address (e.g. ${next_available_address})`"
          class="rounded p-1 border-1 border-gray-100 focus:border-gray-200 outline-none w-full text-xs text-gray-500 grow"
          type="text"
          @change=""/> <!--TODO: update connection address on change  -->
      <div v-if="is_changed_address"
           class="inline-block float-right absolute z-20 right-[3px] top-[-1px]">
        <button
            :disabled="!is_changed_address"
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
          :class="[FIELD_COLOR_LOOKUP[is_changed_endpoint]]" :disabled="peer_local.mobility !== 'static'"
          class="rounded p-1 border-1 border-gray-100 focus:border-gray-200 outline-none w-full text-xs text-gray-500 grow disabled:bg-gray-100"
          placeholder="Endpoint (e.g. 1.2.3.4:51820 example.com:51820)"
          type="text"/>
      <div
          v-if="is_changed_mobility || is_changed_endpoint"
          class="inline-block float-right absolute z-20 right-[3px] top-[-1px]">
        <button
            :disabled="!(is_changed_mobility || is_changed_endpoint)"
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


import WireGuardHelper from "../../js/wg-helper";

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
        changed_fields: {name: null, address: null, mobility: null, endpoint: null},
        errors: {name: null, address: null, mobility: null, endpoint: null},
      },
      FIELD_COLOR_LOOKUP: {
        0: 'bg-white',
        1: 'enabled:bg-green-200',
        '-1': 'enabled:bg-red-200',
      },
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
      if (this.peer_local[field_name] === this.peer[field_name]) return 0;
      if (!WireGuardHelper.checkField(field_name, this.peer_local[field_name])) return -1;
      return 1;
    },
    emit_island_change_sum() {
      this.$emit("updated-change-sum", this.island_change_sum);
    }
  },
  computed: {
    is_changed_name() {
      const field_status = this.check_field_status('name');
      this.island_change_sum.errors.name = field_status === -1 ? 'name cannot be empty' : null;
      this.island_change_sum.changed_fields.name = field_status === 1 ? this.peer_local.name : null;
      this.emit_island_change_sum();
      return field_status;
    },
    is_changed_address() {
      const field_status = this.check_field_status('address');
      this.island_change_sum.errors.address = field_status === -1 ? 'address is not IPv4' : null;
      this.island_change_sum.changed_fields.address = field_status === 1 ? this.peer_local.address : null;
      this.emit_island_change_sum();
      return field_status;
    },
    is_changed_mobility() {
      const field_status = this.check_field_status('mobility');
      this.island_change_sum.errors.mobility = field_status === -1 ? 'mobility is invalid' : null;
      this.island_change_sum.changed_fields.mobility = field_status === 1 ? this.peer_local.mobility : null;
      this.emit_island_change_sum();
      return field_status;
    },
    is_changed_endpoint() {
      const field_status = this.check_field_status('endpoint');
      this.island_change_sum.errors.endpoint = field_status === -1 ? 'endpoint is not IPv4' : null;
      this.island_change_sum.changed_fields.endpoint = field_status === 1 ? this.peer_local.endpoint : null;
      this.emit_island_change_sum();
      return field_status;
    },
    color_div() {
      let changeDetected = false;
      for (const field_status of [this.is_changed_name, this.is_changed_address, this.is_changed_mobility, this.is_changed_endpoint]) {
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