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

          <!-- New Connection Default -->
          <h5 class="text-2xl mt-5 mb-2 pl-2">New Connection Defaults</h5>

          <!-- Persistent Keepalive -->
          <persistent-keepalive-island
              :default-persistent-keepalive="{enabled: false, period: 0}"
              :connection="{persistent_keepalive: defaults_local.connection.persistent_keepalive}"
              :is-new-connection="false"
              class="my-2 mr-2"
              @updated-change-sum="onUpdatedPersistentKeepaliveChangeSum"></persistent-keepalive-island>
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
import PersistentKeepaliveIsland from "@/src/components/islands/persistent-keepalive.vue";
import CompareButton from "@/src/components/ui/buttons/compare.vue";
import EditButton from "@/src/components/ui/buttons/edit.vue";

export default {
  name: "network-defaults-dialog",
  components: {
    CustomDialog,
    ChangeSum,
    PeerKindIconIsland,
    DnsmtuIsland,
    ScriptsIsland,
    PersistentKeepaliveIsland,
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
      peerKindIconIslandChangeSum: null,
      dnsmtuIslandChangeSum: null,
      scriptsIslandChangeSum: null,
      persistentKeepaliveIslandChangeSum: null
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
    onUpdatedKindIconChangeSum(data) {
      this.peerKindIconIslandChangeSum = data;
    },
    onUpdatedDnsMtuChangeSum(data) {
      this.dnsmtuIslandChangeSum = data;
    },
    onUpdatedScriptsChangeSum(data) {
      this.scriptsIslandChangeSum = data;
    },
    onUpdatedPersistentKeepaliveChangeSum(data) {
      this.persistentKeepaliveIslandChangeSum = data;
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
          defaults: {
            peer: {},
            connection: {}
          }
        },
        changed_fields: {
          defaults: {
            peer: {},
            connection: {}
          }
        }
      };

      // Combine peer island changeSums
      for (const island_datum of [this.peerKindIconIslandChangeSum, this.dnsmtuIslandChangeSum, this.scriptsIslandChangeSum]) {
        if (!island_datum) continue;

        // Merge errors
        for (const [field, value] of Object.entries(island_datum.errors)) {
          if (field === "scripts" && value) {
            data.errors.defaults.peer.scripts = {};
            for (const [script_field, script_value] of Object.entries(value)) {
              if (script_value) data.errors.defaults.peer.scripts[script_field] = script_value;
            }
            if (Object.keys(data.errors.defaults.peer.scripts).length === 0) {
              delete data.errors.defaults.peer.scripts;
            }
          } else {
            if (value) data.errors.defaults.peer[field] = value;
          }
        }

        // Merge into defaults.peer
        for (const [field, value] of Object.entries(island_datum.changed_fields)) {
          if (field === "scripts" && value) {
            data.changed_fields.defaults.peer.scripts = {};
            for (const [script_field, script_value] of Object.entries(value)) {
              if (script_value) data.changed_fields.defaults.peer.scripts[script_field] = script_value;
            }
            if (Object.keys(data.changed_fields.defaults.peer.scripts).length === 0) {
              delete data.changed_fields.defaults.peer.scripts;
            }
          } else {
            if (value) data.changed_fields.defaults.peer[field] = value;
          }
        }
      }
      if (Object.keys(data.errors.defaults.peer).length === 0) delete data.errors.defaults.peer;
      if (Object.keys(data.changed_fields.defaults.peer).length === 0) delete data.changed_fields.defaults.peer;

      // Add connection island changeSums
      if (this.persistentKeepaliveIslandChangeSum) {
        for (const [field, value] of Object.entries(this.persistentKeepaliveIslandChangeSum.errors)) {
          if (value) data.errors.defaults.connection[field] = value;
        }
        for (const [field, value] of Object.entries(this.persistentKeepaliveIslandChangeSum.changed_fields)) {
          if (value) data.changed_fields.defaults.connection[field] = value;
        }
      }
      if (Object.keys(data.errors.defaults.connection).length === 0) delete data.errors.defaults.connection;
      if (Object.keys(data.changed_fields.defaults.connection).length === 0) delete data.changed_fields.defaults.connection;

      return data;
    },
    errorDetected() {
      return !!(Object.keys(this.changeSum.errors.defaults).length +
          Object.keys(this.changeSum.errors.defaults).length);
    },
    changeDetected() {
      return !!(Object.keys(this.changeSum.changed_fields.defaults).length +
          Object.keys(this.changeSum.changed_fields.defaults).length);
    }
  }
}
</script>
