<template>
  <div>
    <!-- Dialog: Amnezia Parameters Settings -->
    <custom-dialog :left-button-click="() => { $emit('update:dialogId', ''); }"
                   :left-button-text="'Cancel'"
                   right-button-color="green"
                   :right-button-click="() => { overlayDialogId = 'confirm-changes' }"
                   :right-button-text="'Save Configuration'"
                   :rightButtonDisabled="!changeDetected || errorDetected"
                   class="z-10">

      <!-- title and top bar -->
      <div class="flex flex-col items-center">
        <h3 class="text-3xl leading-tight font-medium text-gray-900 inline mb-5 text-start w-full">
          Amnezia WireGuard Parameters
        </h3>
        <p class="text-sm text-gray-600 mb-3 w-full">
          Configure AmneziaWG parameters for advanced obfuscation.
        </p>
        <span class="order-last w-full flex justify-evenly p-1 px-0 md:px-2 mb-1 mr-2">
          <compare-button :active="page === 'view-changes'"
                          :disabled="!(changeDetected || errorDetected)"
                          image-classes="h-10 w-10"
                          title="See the configuration differences"
                          @click="page = 'view-changes'"></compare-button>
          <edit-button :active="page === 'edit'"
                       image-classes="h-10 w-10"
                       title="Edit Amnezia parameters"
                       @click="page = 'edit'"></edit-button>
        </span>
      </div>

      <div class="flex max-h-[calc(100vh-20rem)] flex-col overflow-y-auto">
        <!-- edit config -->
        <div v-show="page === 'edit'" class="mt-0 w-full overflow-scroll text-start">

          <!-- Network-level Amnezia Parameters -->
          <network-amnezia-params-island
              :network="{amnezia_parameters: amnezia_local.network}"
              :is-new-network="false"
              class="my-2 mr-2"
              @updated-change-sum="onUpdatedNetworkAmneziaChangeSum"></network-amnezia-params-island>

          <!-- Default Peer-level Amnezia Parameters -->
          <h5 class="text-2xl mt-3 mb-2 pl-2">New Peer Amnezia Parameter Defaults</h5>
          <peer-amnezia-params-island
              :peer="{amnezia_parameters: amnezia_local.peer}"
              :default-amnezia-parameters="{jc: 0, jmin: 0, jmax: 0}"
              :is-new-peer="false"
              class="my-2 mr-2"
              @updated-change-sum="onUpdatedPeerAmneziaChangeSum"></peer-amnezia-params-island>

        </div>

        <!-- view changes -->
        <div v-show="page === 'view-changes'" class="mt-2 w-full overflow-scroll text-start">
          <change-sum :change-sum="changeSum" :network="network"></change-sum>
        </div>
      </div>

    </custom-dialog>

    <!-- Dialog: Confirm Changes-->
    <custom-dialog v-if="overlayDialogId === 'confirm-changes'"
                   :left-button-click="() => { overlayDialogId = ''; }"
                   :left-button-text="'Cancel'"
                   right-button-color="green"
                   :right-button-click="() => { updateConfiguration(); overlayDialogId = ''; $emit('update:dialogId', ''); }"
                   :right-button-text="'Do it!'"
                   class="z-20"
                   icon="danger">
      <h3 class="text-lg leading-6 font-medium text-gray-900">
        Confirm changes to Amnezia Parameters
      </h3>
      <div class="my-2 text-sm text-gray-500">
        Are you sure you want to make these changes? Incorrect parameters may prevent WireGuard from functioning.
      </div>

      <div class="flex max-h-[calc(100vh-20rem)] flex-col overflow-y-auto">
        <change-sum :change-sum="changeSum" :network="network"></change-sum>
      </div>
    </custom-dialog>

  </div>
</template>

<script>
import CustomDialog from "@/src/components/dialogs/custom-dialog.vue";
import ChangeSum from "@/src/components/change-sum.vue";
import NetworkAmneziaParamsIsland from "@/src/components/islands/network-amnezia-params.vue";
import PeerAmneziaParamsIsland from "@/src/components/islands/peer-amnezia-params.vue";
import CompareButton from "@/src/components/ui/buttons/compare.vue";
import EditButton from "@/src/components/ui/buttons/edit.vue";

export default {
  name: "amnezia-parameters-dialog",
  components: {
    CustomDialog,
    ChangeSum,
    NetworkAmneziaParamsIsland,
    PeerAmneziaParamsIsland,
    CompareButton,
    EditButton
  },
  props: {
    api: {
      type: Object,
      required: true
    },
    network: {
      type: Object,
      required: true
    },
    dialogId: {
      type: String,
      default: ''
    }
  },
  emits: ['update:dialogId'],
  data() {
    return {
      page: 'edit',
      overlayDialogId: '',
      amnezia_local: {
        network: {
          enabled: false,
          s1: 0,
          s2: 0,
          h1: 0,
          h2: 0,
          h3: 0,
          h4: 0
        },
        peer: {
          jc: 0,
          jmin: 0,
          jmax: 0
        }
      },
      networkAmneziaIslandChangeSum: null,
      peerAmneziaIslandChangeSum: null
    };
  },
  created() {
    // Initialize with current network amnezia parameters
    this.amnezia_local.network = JSON.parse(JSON.stringify(this.network.amnezia_parameters));
    // Initialize with default peer amnezia parameters
    this.amnezia_local.peer = JSON.parse(JSON.stringify(this.network.defaults.peer.amnezia_parameters));
  },
  methods: {
    onUpdatedNetworkAmneziaChangeSum(data) {
      this.networkAmneziaIslandChangeSum = data;
    },
    onUpdatedPeerAmneziaChangeSum(data) {
      this.peerAmneziaIslandChangeSum = data;
    },
    async updateConfiguration() {
      try {
        await this.api.patch_network_config(this.changeSum);
        // Refresh the network data
        location.reload();
      } catch (err) {
        alert(`Failed to update configuration: ${err.message || err}`);
        console.error(err);
      }
    }
  },
  computed: {
    changeSum() {
      const data = {
        errors: {
          amnezia_parameters: {},
          defaults: {
            peer: {},
          }
        },
        changed_fields: {
          amnezia_parameters: {},
          defaults: {
            peer: {},
          }
        }
      };

      // Add network amnezia changes
      if (this.networkAmneziaIslandChangeSum) {
        for (const [field, value] of Object.entries(this.networkAmneziaIslandChangeSum.errors)) {
          if (value) data.errors.amnezia_parameters[field] = value;
        }
        for (const [field, value] of Object.entries(this.networkAmneziaIslandChangeSum.changed_fields)) {
          if (value) data.changed_fields.amnezia_parameters[field] = value;
        }
      }

      // Add peer amnezia changes (goes into defaults.peer)
      if (this.peerAmneziaIslandChangeSum) {
        for (const [field, value] of Object.entries(this.peerAmneziaIslandChangeSum.errors)) {
          if (value) data.errors.defaults.peer[field] = value;
        }
        for (const [field, value] of Object.entries(this.peerAmneziaIslandChangeSum.changed_fields)) {
          if (value) data.changed_fields.defaults.peer[field] = value;
        }
      }
      if (Object.keys(data.errors.defaults.peer).length === 0) delete data.errors.defaults.peer;
      if (Object.keys(data.changed_fields.defaults.peer).length === 0) delete data.changed_fields.defaults.peer;

      return data;
    },
    errorDetected() {
      return !!(Object.keys(this.changeSum.errors.amnezia_parameters).length +
          Object.keys(this.changeSum.errors.defaults).length);
    },
    changeDetected() {
      return !!(Object.keys(this.changeSum.changed_fields.amnezia_parameters).length +
          Object.keys(this.changeSum.changed_fields.defaults).length);
    }
  }
}
</script>
