<template>

  <div :class="[peerEditConfigColor]" class="my-2 mr-2 p-1 shadow-md border rounded">
    <div class="my-0.5 truncate flex items-center relative">
                                   <span class="text-gray-800 text-xs mr-1">
                                     <strong class="text-sm">Name:</strong>
                                   </span>
      <input
          v-model="peer_local.name"
          :class="[peerEditNameColor]"
          class="rounded p-1 border-1 border-gray-100 focus:border-gray-200 outline-none w-full text-xs text-gray-500 grow"
          placeholder="Name" type="text"/>
      <div v-if="peerEditNameColor !== 'bg-white'"
           class="inline-block float-right absolute z-20 right-[3px] top-[-1px]">
        <button
            :disabled="peerEditNameColor === 'bg-white'"
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
          :class="[peerEditAddressColor]"
          :placeholder="`Address (e.g. ${next_available_address})`"
          class="rounded p-1 border-1 border-gray-100 focus:border-gray-200 outline-none w-full text-xs text-gray-500 grow"
          type="text"
          @change=""/> <!--TODO: update connection address on change  -->
      <div v-if="peerEditAddressColor !== 'bg-white'"
           class="inline-block float-right absolute z-20 right-[3px] top-[-1px]">
        <button
            :disabled="peerEditAddressColor === 'bg-white'"
            class="align-middle p-0.5 rounded bg-gray-100 hover:bg-gray-500 hover:text-white transition undo-button-itself"
            title="Undo Changes"
            @click="peer_local.address = peer.address">
          <img alt="Undo" class="h-4" src="../../icons/flowbite/undo.svg"/>
        </button>
      </div>
    </div>

    <div class="form-check truncate flex items-center relative">
      <label class="flex-none">
        <input
            :checked="peer_local.mobility === 'static'"
            class="form-check-input appearance-none h-4 w-4 border border-gray-300 rounded-sm bg-white checked:bg-blue-600 checked:border-blue-600 focus:outline-none transition duration-200 mt-1 align-top bg-no-repeat bg-center bg-contain float-left mr-1 cursor-pointer"
            type="checkbox"
            @change="peer_local.mobility = peer_local.mobility === 'static' ? 'roaming' : 'static';">
        <span class="text-gray-800 cursor-pointer text-xs mr-1">
                <strong class="text-sm">Static Endpoint:</strong>
              </span>
      </label>

      <input
          v-model="peer_local.endpoint"
          :class="[peerEditEndpointColor]" :disabled="peer_local.mobility !== 'static'"
          class="rounded p-1 border-1 border-gray-100 focus:border-gray-200 outline-none w-full text-xs text-gray-500 grow"
          placeholder="Endpoint (e.g. 1.2.3.4:51820 example.com:51820)"
          type="text"/>
      <div
          v-if="!(peer_local.endpoint === peer.endpoint && peer_local.mobility === peer.mobility)"
          class="inline-block float-right absolute z-20 right-[3px] top-[-1px]">
        <button
            :disabled="peer_local.endpoint === peer.endpoint && peer_local.mobility === peer.mobility"
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
    value: {
      type: Object,
      default: {
        changedFields: {},
        error: null,
      },
    },
  },
  data() {
    return {
      peer_local: {},
      PEER_SUMMARY_KEY_LOOKUP: {
        name: 'Name',
        address: 'Address',
        endpoint: 'Endpoint',
      },
    };
  },
  created() {
    this.peer_local = this.peer;
  },
  emits: ['update:value'],
  computed: {
    peerEditNameColor() {
      // eslint-disable-next-line no-nested-ternary
      return this.peer_local.name !== this.peer.name
          ? (WireGuardHelper.checkField('name', this.peer_local.name) ? 'bg-green-200' : 'bg-red-200') : 'bg-white';
    },
    peerEditAddressColor() {
      // eslint-disable-next-line no-nested-ternary
      return this.peer_local.address !== this.peer.address
          ? (WireGuardHelper.checkField('address', this.peer_local.address) ? 'bg-green-200' : 'bg-red-200') : 'bg-white';
    },
    peerEditEndpointColor() {
      // eslint-disable-next-line no-nested-ternary
      return this.peer_local.mobility === 'static' ? this.peer_local.endpoint !== this.peer.endpoint || !WireGuardHelper.checkField('endpoint', this.peer_local.endpoint)
          ? (WireGuardHelper.checkField('endpoint', this.peer_local.endpoint) ? 'bg-green-200' : 'bg-red-200') : 'bg-white' : 'bg-gray-100';
    },
    peerEditConfigColor() {
      let error = false;
      let changeDetected = false;
      error ||= this.peerEditNameColor === 'bg-red-200';
      changeDetected ||= this.peerEditNameColor === 'bg-green-200';
      error ||= this.peerEditAddressColor === 'bg-red-200';
      changeDetected ||= this.peerEditAddressColor === 'bg-green-200';
      error ||= this.peerEditEndpointColor === 'bg-red-200';
      changeDetected ||= this.peerEditEndpointColor === 'bg-green-200';
      // eslint-disable-next-line no-nested-ternary
      return error ? 'bg-red-50' : changeDetected ? 'bg-green-100' : 'bg-green-50';
    },
  },
}
</script>

<style scoped>

</style>