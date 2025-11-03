<template>

  <div :class="[this.are_keys_updated ? 'bg-green-100' : 'bg-green-50']"
      class="my-2 pt-1 pb-2 px-3 shadow-md border rounded relative">
    <div class="overflow-x-auto text-lg whitespace-nowrap">
      <div class="mt-1 flex items-center">
        <field class="inline-block" field="PublicKey  :"></field>
        <refresh-button title="Refresh Public/Private Keys" @click="refreshPeerEditKeys()"></refresh-button>
        <span class="text-gray-800">{{ peer_local_public_key }}</span>
      </div>
      <div class="mt-1 flex items-center">
        <field class="inline-block" field="PrivateKey:"></field>
        <refresh-button title="Refresh Public/Private Keys" @click="refreshPeerEditKeys()"></refresh-button>
        <span class="text-gray-800">{{ peer_local_private_key }}</span>
      </div>
      <div v-show="peer.created_at" class="mt-1">
        <field class="inline-block" field="CreatedAt  :"></field>
        <div class="text-gray-800 ml-2 inline-block">
          {{ new Date(peer.created_at).toString() }}
        </div>
      </div>
      <div v-show="peer.updated_at" class="mt-1">
        <field class="inline-block" field="UpdatedAt  :"></field>
        <div class="text-gray-800 ml-2 inline-block">
          {{ new Date(peer.updated_at).toString() }}
        </div>
      </div>
    </div>

    <!-- Undo Button -->
    <undo-button v-if="are_keys_updated"
                 :disabled="!are_keys_updated"
                 alignment-classes="right-[6px] top-[6px]"
                 image-classes="h-7"
                 @click="peer_local_private_key = peer.private_key; $emit('updated-change-sum', {changed_fields: {}, errors: {}},
    )">
    </undo-button>
  </div>

</template>

<script>
import Field from "@/src/components/ui/field.vue";
import UndoButton from "@/src/components/ui/buttons/undo.vue";
import RefreshButton from "@/src/components/ui/buttons/refresh.vue";
import WireGuardHelper from "@/src/js/wg-helper.js";

export default {
  name: "peer-details-island",
  components: {RefreshButton, UndoButton, Field},
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
      peer_local_private_key: {},
    };
  },
  created() {
    this.peer_local_private_key = this.peer.private_key;
  },
  emits: ['updated-change-sum'],
  methods: {
    async refreshPeerEditKeys() {
      this.peer_local_private_key = WireGuardHelper.wg_generate_key();
      this.$emit("updated-change-sum", {
            changed_fields: {
              private_key: this.peer_local_private_key,
            },
            errors: {},
          },
      );
    }
  },
  computed: {
    are_keys_updated() {
      return this.peer_local_private_key !== this.peer.private_key;
    },
    peer_local_public_key() {
      return WireGuardHelper.wg_public_key_from_private_key(this.peer_local_private_key);
    }
  },
}
</script>

<style scoped>

</style>