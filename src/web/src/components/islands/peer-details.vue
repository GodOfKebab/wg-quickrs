<template>

  <div
      :class="[this.are_keys_updated ? 'bg-green-100' : 'bg-green-50']"
      class="my-2 p-2 px-3 shadow-md border rounded relative">
    <div
        v-if="are_keys_updated"
        class="inline-block float-right absolute z-20 right-[0.2rem] top-[0rem]">
      <button
          :disabled="!are_keys_updated"
          class="align-middle p-0.5 rounded bg-gray-100 hover:bg-gray-500 hover:text-white transition"
          title="Undo Changes"
          @click="peer_local.public_key = peer.public_key; peer_local.private_key = peer.private_key">
        <img alt="Undo" class="h-4" src="/icons/flowbite/undo.svg"/>
      </button>
    </div>
    <div class="overflow-x-auto">
      <div v-for="peerConfigKey in ['public_key', 'private_key', 'created_at', 'updated_at']">
        <div v-if="peer_local[peerConfigKey]" class="grid grid-cols-9 text-sm refresh-key">
          <div class="col-span-2 mb-0.5">
            <span class="text-gray-800">
              <strong>{{ PEER_DETAILS_KEY_LOOKUP[peerConfigKey] }}</strong>
            </span>
          </div>
          <div class="col-span-7 text-sm whitespace-nowrap">
            <span class="text-gray-800 text-xs mr-1">:</span>
            <div v-if="['created_at', 'updated_at'].includes(peerConfigKey)"
                 class="text-gray-800 text-xs pr-4 inline-block">
              {{ new Date(peer_local[peerConfigKey]).toString() }}
            </div>
            <div v-else class="pr-4 inline-block">
              <button class="align-middle rounded bg-gray-100 hover:bg-gray-600 hover:text-white transition-all mr-1"
                      @click="refreshPeerEditKeys()">
                <img alt="Refresh Keys" class="h-4" src="/icons/flowbite/refresh.svg"/>
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
    api: {
      type: Object,
      default: null,
    }
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
    this.peer_local = JSON.parse(JSON.stringify(this.peer));
  },
  emits: ['updated-change-sum'],
  methods: {
    async refreshPeerEditKeys() {
      await this.api.get_wireguard_public_private_keys().then(response => {
        this.peer_local.public_key = response.public_key;
        this.peer_local.private_key = response.private_key;
        this.$emit("updated-change-sum", {
              changed_fields: response,
              errors: {},
            },
        );
      });
    }
  },
  computed: {
    are_keys_updated() {
      return this.peer_local.public_key !== this.peer.public_key || this.peer_local.private_key !== this.peer.private_key;
    },
  },
}
</script>

<style scoped>

</style>