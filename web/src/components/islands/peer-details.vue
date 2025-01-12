<template>

  <div
      :class="[computed_colors.div]"
      class="p-1 shadow-md border rounded relative">
    <div
        v-if="are_keys_updated"
        class="inline-block float-right absolute z-20 right-[0.2rem] top-[0rem]">
      <button
          :disabled="!are_keys_updated"
          class="align-middle p-0.5 rounded bg-gray-100 hover:bg-gray-500 hover:text-white opacity-0 transition undo-button-itself"
          title="Undo Changes"
          @click="peer_local.public_key = peer.public_key; peer_local.private_key = peer.private_key">
        <img alt="Undo" class="h-4" src="../../icons/flowbite/undo.svg"/>
      </button>
    </div>
    <div class="overflow-x-auto">
      <div v-for="peerConfigKey in ['public_key', 'private_key', 'created_at', 'updated_at']">
        <div class="grid grid-cols-10 text-sm refresh-key">
          <div class="col-span-2">
                                 <span class="text-gray-800">
                                     <strong>{{ PEER_DETAILS_KEY_LOOKUP[peerConfigKey] }}</strong>
                                 </span>
          </div>
          <div class="col-span-8 text-sm whitespace-nowrap">
            <span class="text-gray-800 text-xs mr-1">:</span>
            <div v-if="['created_at', 'updated_at'].includes(peerConfigKey)"
                 class="text-gray-800 text-xs pr-4 inline-block">
              {{ new Date(peer_local[peerConfigKey]).toString() }}
            </div>
            <div v-else class="pr-4 inline-block">
              <button class="align-middle rounded bg-gray-100 hover:bg-gray-600 hover:text-white transition-all mr-1"
                      @click="refreshPeerEditKeys()">
                <img alt="Refresh Keys" class="h-4" src="../../icons/flowbite/refresh.svg"/>
              </button>
              <span v-if="peerConfigKey === 'public_key'" class="text-gray-800 text-xs">{{
                  peer_local.public_key
                }}</span>
              <span v-else class="text-gray-800 text-xs">{{ peer_local.private_key }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>

</template>

<script>


export default {
  name: "peer-details-island",
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
      PEER_DETAILS_KEY_LOOKUP: {
        public_key: 'PublicKey',
        private_key: 'PrivateKey',
        created_at: 'CreatedAt',
        updated_at: 'UpdatedAt',
      },
    };
  },
  created() {
    this.peer_local = this.peer;
  },
  emits: ['update:value'],
  computed: {
    are_keys_updated() {
      return this.peer_local.public_key !== this.peer.public_key || this.peer_local.private_key !== this.peer.private_key;
    },
    computed_colors() {
      const changedFields = {public_key: "", private_key: ""};
      let error = null;
      const colors = {div: 'bg-green-50 highlight-undo-box'};
      for (const field of ['public_key', 'private_key']) {
        if (this.peer_local[field] !== this.peer[field]) changedFields[field] = this.peer_local[field];
        if (changedFields[field].length === 0) delete changedFields[field];
      }
      if (this.are_keys_updated) colors.div = 'bg-green-100 highlight-undo-box'

      this.value.changedFields = changedFields;
      this.value.error = error;

      return colors;
    },
  },
}
</script>

<style scoped>

</style>