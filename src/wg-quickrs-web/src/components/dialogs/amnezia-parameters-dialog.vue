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
          <div class="my-2 py-2 pl-1 pr-3 shadow-md border rounded bg-purple-50">
            <h5 class="text-2xl mb-3 pl-2">Network-Level Amnezia Parameters</h5>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-3 px-2">

              <!-- Enabled Toggle -->
              <div class="col-span-2 flex items-center gap-2">
                <label class="text-sm font-medium">Enable AmneziaWG:</label>
                <input
                    v-model="amnezia_local.network.enabled"
                    type="checkbox"
                    class="h-4 w-4"
                    @change="onNetworkParamChanged">
              </div>

              <!-- S1 Parameter -->
              <div class="flex flex-col gap-1">
                <label class="text-sm font-medium">S1 (Init Packet Junk Size):</label>
                <input
                    v-model.number="amnezia_local.network.s1"
                    type="number"
                    min="0"
                    max="1132"
                    class="border rounded px-2 py-1"
                    @input="onNetworkParamChanged">
                <span class="text-xs text-gray-500">Range: 0-1132</span>
              </div>

              <!-- S2 Parameter -->
              <div class="flex flex-col gap-1">
                <label class="text-sm font-medium">S2 (Response Packet Junk Size):</label>
                <input
                    v-model.number="amnezia_local.network.s2"
                    type="number"
                    min="0"
                    max="1188"
                    class="border rounded px-2 py-1"
                    @input="onNetworkParamChanged">
                <span class="text-xs text-gray-500">Range: 0-1188, s1+56≠s2</span>
              </div>

              <!-- H1 Parameter -->
              <div class="flex flex-col gap-1">
                <label class="text-sm font-medium">H1 (Init Packet Magic Header):</label>
                <input
                    v-model.number="amnezia_local.network.h1"
                    type="number"
                    min="0"
                    max="4294967295"
                    class="border rounded px-2 py-1"
                    @input="onNetworkParamChanged">
                <span class="text-xs text-gray-500">32-bit unsigned integer</span>
              </div>

              <!-- H2 Parameter -->
              <div class="flex flex-col gap-1">
                <label class="text-sm font-medium">H2 (Response Packet Magic Header):</label>
                <input
                    v-model.number="amnezia_local.network.h2"
                    type="number"
                    min="0"
                    max="4294967295"
                    class="border rounded px-2 py-1"
                    @input="onNetworkParamChanged">
                <span class="text-xs text-gray-500">32-bit unsigned integer</span>
              </div>

              <!-- H3 Parameter -->
              <div class="flex flex-col gap-1">
                <label class="text-sm font-medium">H3 (Underload Packet Magic Header):</label>
                <input
                    v-model.number="amnezia_local.network.h3"
                    type="number"
                    min="0"
                    max="4294967295"
                    class="border rounded px-2 py-1"
                    @input="onNetworkParamChanged">
                <span class="text-xs text-gray-500">32-bit unsigned integer</span>
              </div>

              <!-- H4 Parameter -->
              <div class="flex flex-col gap-1">
                <label class="text-sm font-medium">H4 (Transport Packet Magic Header):</label>
                <input
                    v-model.number="amnezia_local.network.h4"
                    type="number"
                    min="0"
                    max="4294967295"
                    class="border rounded px-2 py-1"
                    @input="onNetworkParamChanged">
                <span class="text-xs text-gray-500">32-bit unsigned integer</span>
              </div>

            </div>
          </div>

          <!-- Default Peer-level Amnezia Parameters -->
          <div class="my-2 py-2 pl-1 pr-3 shadow-md border rounded bg-purple-50">
            <h5 class="text-2xl mb-3 pl-2">New Peer Amnezia Parameter Defaults</h5>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-3 px-2">

              <!-- Jc Parameter -->
              <div class="flex flex-col gap-1">
                <label class="text-sm font-medium">Jc (Junk Packet Count):</label>
                <input
                    v-model.number="amnezia_local.peer.jc"
                    type="number"
                    min="-1"
                    max="128"
                    class="border rounded px-2 py-1"
                    @input="onPeerParamChanged">
                <span class="text-xs text-gray-500">Range: -1 to 128</span>
              </div>

              <!-- Jmin Parameter -->
              <div class="flex flex-col gap-1">
                <label class="text-sm font-medium">Jmin (Min Junk Packet Size):</label>
                <input
                    v-model.number="amnezia_local.peer.jmin"
                    type="number"
                    min="1"
                    max="1279"
                    class="border rounded px-2 py-1"
                    @input="onPeerParamChanged">
                <span class="text-xs text-gray-500">Range: 1-1279</span>
              </div>

              <!-- Jmax Parameter -->
              <div class="flex flex-col gap-1">
                <label class="text-sm font-medium">Jmax (Max Junk Packet Size):</label>
                <input
                    v-model.number="amnezia_local.peer.jmax"
                    type="number"
                    min="1"
                    max="1280"
                    class="border rounded px-2 py-1"
                    @input="onPeerParamChanged">
                <span class="text-xs text-gray-500">Range: 1-1280, jmin&lt;jmax</span>
              </div>

            </div>
          </div>

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
import CompareButton from "@/src/components/ui/buttons/compare.vue";
import EditButton from "@/src/components/ui/buttons/edit.vue";

export default {
  name: "amnezia-parameters-dialog",
  components: {
    CustomDialog,
    ChangeSum,
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
      changeSum: {
        changed_fields: {
          network: {},
          defaults: {}
        },
        errors: {}
      },
      changeDetected: false,
      errorDetected: false
    };
  },
  created() {
    // Initialize with current network amnezia parameters
    this.amnezia_local.network = JSON.parse(JSON.stringify(this.network.amnezia_parameters));
    // Initialize with default peer amnezia parameters
    this.amnezia_local.peer = JSON.parse(JSON.stringify(this.network.defaults.peer.amnezia_parameters));
  },
  methods: {
    onNetworkParamChanged() {
      if (!this.changeSum.changed_fields.network.amnezia_parameters) {
        this.changeSum.changed_fields.network.amnezia_parameters = {};
      }

      const networkParams = this.changeSum.changed_fields.network.amnezia_parameters;
      const originalParams = this.network.amnezia_parameters;

      // Only include changed fields
      if (this.amnezia_local.network.enabled !== originalParams.enabled) {
        networkParams.enabled = this.amnezia_local.network.enabled;
      } else {
        delete networkParams.enabled;
      }

      if (this.amnezia_local.network.s1 !== originalParams.s1) {
        networkParams.s1 = this.amnezia_local.network.s1;
      } else {
        delete networkParams.s1;
      }

      if (this.amnezia_local.network.s2 !== originalParams.s2) {
        networkParams.s2 = this.amnezia_local.network.s2;
      } else {
        delete networkParams.s2;
      }

      if (this.amnezia_local.network.h1 !== originalParams.h1) {
        networkParams.h1 = this.amnezia_local.network.h1;
      } else {
        delete networkParams.h1;
      }

      if (this.amnezia_local.network.h2 !== originalParams.h2) {
        networkParams.h2 = this.amnezia_local.network.h2;
      } else {
        delete networkParams.h2;
      }

      if (this.amnezia_local.network.h3 !== originalParams.h3) {
        networkParams.h3 = this.amnezia_local.network.h3;
      } else {
        delete networkParams.h3;
      }

      if (this.amnezia_local.network.h4 !== originalParams.h4) {
        networkParams.h4 = this.amnezia_local.network.h4;
      } else {
        delete networkParams.h4;
      }

      // Clean up if empty
      if (Object.keys(networkParams).length === 0) {
        delete this.changeSum.changed_fields.network.amnezia_parameters;
      }

      this.updateChangeDetection();
    },
    onPeerParamChanged() {
      if (!this.changeSum.changed_fields.defaults.peer) {
        this.changeSum.changed_fields.defaults.peer = {};
      }
      if (!this.changeSum.changed_fields.defaults.peer.amnezia_parameters) {
        this.changeSum.changed_fields.defaults.peer.amnezia_parameters = {};
      }

      const peerParams = this.changeSum.changed_fields.defaults.peer.amnezia_parameters;
      const originalParams = this.network.defaults.peer.amnezia_parameters;

      // Only include changed fields
      if (this.amnezia_local.peer.jc !== originalParams.jc) {
        peerParams.jc = this.amnezia_local.peer.jc;
      } else {
        delete peerParams.jc;
      }

      if (this.amnezia_local.peer.jmin !== originalParams.jmin) {
        peerParams.jmin = this.amnezia_local.peer.jmin;
      } else {
        delete peerParams.jmin;
      }

      if (this.amnezia_local.peer.jmax !== originalParams.jmax) {
        peerParams.jmax = this.amnezia_local.peer.jmax;
      } else {
        delete peerParams.jmax;
      }

      // Clean up if empty
      if (Object.keys(peerParams).length === 0) {
        delete this.changeSum.changed_fields.defaults.peer.amnezia_parameters;
      }
      if (this.changeSum.changed_fields.defaults.peer && Object.keys(this.changeSum.changed_fields.defaults.peer).length === 0) {
        delete this.changeSum.changed_fields.defaults.peer;
      }

      this.updateChangeDetection();
    },
    updateChangeDetection() {
      const hasNetworkChanges = this.changeSum.changed_fields.network.amnezia_parameters &&
          Object.keys(this.changeSum.changed_fields.network.amnezia_parameters).length > 0;
      const hasPeerChanges = this.changeSum.changed_fields.defaults.peer?.amnezia_parameters &&
          Object.keys(this.changeSum.changed_fields.defaults.peer.amnezia_parameters).length > 0;

      this.changeDetected = hasNetworkChanges || hasPeerChanges;
      this.errorDetected = false; // TODO: Add validation error detection
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
  }
}
</script>
