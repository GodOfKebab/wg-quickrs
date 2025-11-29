<template>
  <div>
    <!-- Dialog: Network Defaults Settings -->
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
          Network Defaults Configuration
        </h3>
        <p class="text-sm text-gray-600 mb-3 w-full">
          Configure network defaults for new peers and connections.
        </p>
        <span class="order-last w-full flex justify-evenly p-1 px-0 md:px-2 mb-1 mr-2">
          <compare-button :active="page === 'view-changes'"
                          :disabled="!(changeDetected || errorDetected)"
                          image-classes="h-10 w-10"
                          title="See the configuration differences"
                          @click="page = 'view-changes'"></compare-button>
          <edit-button :active="page === 'edit'"
                       image-classes="h-10 w-10"
                       title="Edit the default configuration"
                       @click="page = 'edit'"></edit-button>
        </span>
      </div>

      <div class="flex max-h-[calc(100vh-20rem)] flex-col overflow-y-auto">
        <!-- edit config -->
        <div v-show="page === 'edit'" class="mt-0 w-full overflow-scroll text-start">

          <!-- New Peer Defaults -->
          <div class="my-2 py-2 pl-2 pr-1 shadow-md border rounded bg-purple-50">
            <h5 class="text-2xl mb-3 pl-2">New Peer Defaults</h5>

            <!-- New Peer Default Kind & Icon -->
            <peer-kind-icon-island
                :default-kind-icon="{kind: '', icon: {enabled: false, src: ''}}"
                :peer="{kind: defaults_local.peer.kind, icon: defaults_local.peer.icon}"
                class="my-2 mr-2"
                @updated-change-sum="onUpdatedKindIconChangeSum"></peer-kind-icon-island>

            <!-- New Peer Default DNS & MTU -->
            <dnsmtu-island
                :default-dnsmtu="{dns: {enabled: false, addresses: []}, mtu: {enabled: false, value: 0}}"
                :peer="{dns: defaults_local.peer.dns, mtu: defaults_local.peer.mtu}"
                class="my-2 mr-2"
                @updated-change-sum="onUpdatedDnsMtuChangeSum"></dnsmtu-island>

            <!-- New Peer Default Scripts -->
            <scripts-island
                :default-scripts="{pre_up: [], post_up: [], pre_down: [], post_down: []}"
                :peer="{scripts: defaults_local.peer.scripts}"
                :is-this-peer="false"
                class="my-2 mr-2"
                @updated-change-sum="onUpdatedScriptsChangeSum"></scripts-island>
          </div>

          <!-- New Connection Default -->
          <div class="my-2 py-2 pl-1 pr-3 shadow-md border rounded bg-purple-50">
            <h5 class="text-2xl mb-2 pl-2">New Connection Defaults</h5>

            <!-- Persistent Keepalive -->
            <div class="w-92">
              <input-field v-model="defaults_local.connection.persistent_keepalive.period"
                           input-color="bg-white"
                           value-field="period"
                           :value-prev="defaults_local.connection.persistent_keepalive.period"
                           undo-button-alignment-classes="right-[6px] top-[4px]"
                           label="PersistentKeepalive"
                           placeholder="seconds"></input-field>
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
        Confirm changes to Network Defaults
      </h3>
      <div class="my-2 text-sm text-gray-500">
        Are you sure you want to make these changes?
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
import PeerKindIconIsland from "@/src/components/islands/peer-kind-icon.vue";
import DnsmtuIsland from "@/src/components/islands/dnsmtu.vue";
import ScriptsIsland from "@/src/components/islands/scripts.vue";
import CompareButton from "@/src/components/ui/buttons/compare.vue";
import EditButton from "@/src/components/ui/buttons/edit.vue";
import InputField from "@/src/components/ui/input-field.vue";

export default {
  name: "network-defaults-dialog",
  components: {
    InputField,
    CustomDialog,
    ChangeSum,
    PeerKindIconIsland,
    DnsmtuIsland,
    ScriptsIsland,
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
      defaults_local: {
        peer: {
          kind: '',
          icon: {enabled: false, src: ''},
          dns: {enabled: false, addresses: []},
          mtu: {enabled: false, value: 0},
          scripts: {pre_up: [], post_up: [], pre_down: [], post_down: []},
          amnezia_parameters: {jc: 0, jmin: 0, jmax: 0}
        },
        connection: {
          persistent_keepalive: {enabled: false, period: 0}
        }
      },
      changeSum: {
        changed_fields: {
          defaults: {}
        },
        errors: {}
      },
      changeDetected: false,
      errorDetected: false
    };
  },
  created() {
    // Initialize with current defaults
    this.defaults_local.peer.kind = this.network.defaults.peer.kind;
    this.defaults_local.peer.icon = JSON.parse(JSON.stringify(this.network.defaults.peer.icon));
    this.defaults_local.peer.dns = JSON.parse(JSON.stringify(this.network.defaults.peer.dns));
    this.defaults_local.peer.mtu = JSON.parse(JSON.stringify(this.network.defaults.peer.mtu));
    this.defaults_local.peer.scripts = JSON.parse(JSON.stringify(this.network.defaults.peer.scripts));
    this.defaults_local.peer.amnezia_parameters = JSON.parse(JSON.stringify(this.network.defaults.peer.amnezia_parameters));
    this.defaults_local.connection.persistent_keepalive = JSON.parse(JSON.stringify(this.network.defaults.connection.persistent_keepalive));
  },
  methods: {
    onUpdatedKindIconChangeSum(changeSum) {
      if (changeSum.changed_fields?.peers) {
        const peerChanges = Object.values(changeSum.changed_fields.peers)[0];
        if (!this.changeSum.changed_fields.defaults.peer) {
          this.changeSum.changed_fields.defaults.peer = {};
        }
        if (peerChanges.kind !== undefined) {
          this.changeSum.changed_fields.defaults.peer.kind = peerChanges.kind;
        }
        if (peerChanges.icon !== undefined) {
          this.changeSum.changed_fields.defaults.peer.icon = peerChanges.icon;
        }
      }
      this.updateChangeDetection();
    },
    onUpdatedDnsMtuChangeSum(changeSum) {
      if (changeSum.changed_fields?.peers) {
        const peerChanges = Object.values(changeSum.changed_fields.peers)[0];
        if (!this.changeSum.changed_fields.defaults.peer) {
          this.changeSum.changed_fields.defaults.peer = {};
        }
        if (peerChanges.dns !== undefined) {
          this.changeSum.changed_fields.defaults.peer.dns = peerChanges.dns;
        }
        if (peerChanges.mtu !== undefined) {
          this.changeSum.changed_fields.defaults.peer.mtu = peerChanges.mtu;
        }
      }
      this.updateChangeDetection();
    },
    onUpdatedScriptsChangeSum(changeSum) {
      if (changeSum.changed_fields?.peers) {
        const peerChanges = Object.values(changeSum.changed_fields.peers)[0];
        if (!this.changeSum.changed_fields.defaults.peer) {
          this.changeSum.changed_fields.defaults.peer = {};
        }
        if (peerChanges.scripts !== undefined) {
          this.changeSum.changed_fields.defaults.peer.scripts = peerChanges.scripts;
        }
      }
      this.updateChangeDetection();
    },
    onConnectionKeepaliveChanged() {
      if (!this.changeSum.changed_fields.defaults.connection) {
        this.changeSum.changed_fields.defaults.connection = {};
      }
      this.changeSum.changed_fields.defaults.connection.persistent_keepalive =
        this.defaults_local.connection.persistent_keepalive;
      this.updateChangeDetection();
    },
    updateChangeDetection() {
      this.changeDetected = Object.keys(this.changeSum.changed_fields.defaults).length > 0;
      this.errorDetected = false; // TODO: Add error detection
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
