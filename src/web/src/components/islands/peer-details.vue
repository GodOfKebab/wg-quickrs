<template>

  <div :class="[this.are_keys_updated ? 'bg-green-100' : 'bg-green-50']"
      class="my-2 p-2 px-3 shadow-md border rounded relative">
    <div class="overflow-x-auto text-lg whitespace-nowrap">
      <div class="mb-1">
        <field class="inline-block" field="PublicKey  :"></field>
        <button
            class="align-middle rounded bg-gray-100 hover:bg-gray-600 hover:text-white transition-all mx-2 cursor-pointer"
            @click="refreshPeerEditKeys()">
          <img alt="Refresh Keys" class="h-6" src="/icons/flowbite/refresh.svg"/>
        </button>
        <span class="text-gray-800">{{ peer_local.public_key }}</span>
      </div>
      <div class="mb-1">
        <field class="inline-block" field="PrivateKey:"></field>
        <button
            class="align-middle rounded bg-gray-100 hover:bg-gray-600 hover:text-white transition-all mx-2 cursor-pointer"
            @click="refreshPeerEditKeys()">
          <img alt="Refresh Keys" class="h-6" src="/icons/flowbite/refresh.svg"/>
        </button>
        <span class="text-gray-800">{{ peer_local.private_key }}</span>
      </div>
      <div class="mb-1">
        <field class="inline-block" field="CreatedAt  :"></field>
        <div class="text-gray-800 ml-2 inline-block">
          {{ new Date(peer_local.created_at).toString() }}
        </div>
      </div>
      <div>
        <field class="inline-block" field="UpdatedAt  :"></field>
        <div class="text-gray-800 ml-2 inline-block">
          {{ new Date(peer_local.updated_at).toString() }}
        </div>
      </div>
    </div>

    <!-- Undo Button -->
    <undo-button v-if="are_keys_updated"
                 :disabled="!are_keys_updated"
                 alignment-classes="right-[6px] top-[6px]"
                 image-classes="h-7"
                 @click="peer_local.public_key = peer.public_key; peer_local.private_key = peer.private_key;">
    </undo-button>
  </div>

</template>

<script>
import Field from "@/components/ui/field.vue";
import UndoButton from "@/components/ui/undo-button.vue";

export default {
  name: "peer-details-island",
  components: {UndoButton, Field},
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