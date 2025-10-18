<template>

  <div>
    <!-- Dialog: Settings -->
    <custom-dialog :left-button-click="() => { $emit('update:dialogId', ''); }"
                   :left-button-text="'Cancel'"
                   :right-button-classes="['enabled:bg-green-700', 'enabled:hover:bg-green-800', 'enabled:border-green-900', 'enabled:focus:outline-none', 'bg-gray-200', 'disabled:hover:bg-gray-200', 'disabled:cursor-not-allowed', 'disabled:border-gray-200', 'text-white']"
                   :right-button-click="() => { overlayDialogId = 'confirm-changes'; }"
                   :rightButtonDisabled="page !== 'file' && errorDetected"
                   class="z-10"
                   right-button-text="Create Peer">

      <!-- title and top bar -->
      <div class="flex flex-col items-center">
        <h3 class="text-3xl leading-6 font-medium text-gray-900 inline mb-5 text-start w-full">
          Create a new Peer:
        </h3>
        <span class="order-last w-full flex justify-between px-1 mb-1">
          <delete-button disabled="true"
                         title="Delete this peer"
                         image-classes="h-10 w-10"></delete-button>
          <compare-button :active="page === 'view-changes'"
                          image-classes="h-10 w-10"
                          title="See the configuration differences for this peer"
                          @click="page = 'view-changes'"></compare-button>
          <edit-button :active="page === 'edit'"
                       image-classes="h-10 w-10"
                       title="Edit the configuration for this peer"
                       @click="page = 'edit'"></edit-button>
          <conf-button disabled="true"
                       image-classes="h-10 w-10"
                       title="See the configuration file for this peer"></conf-button>
          <qr-button disabled="true"
                     image-classes="h-10 w-10"
                     title="Show QR Code"></qr-button>
          <download-button disabled="true"
                           image-classes="h-10 w-10"
                           title="Download Configuration"></download-button>
        </span>
      </div>

      <div class="flex max-h-[calc(100vh-20rem)] flex-col overflow-y-auto">
        <!-- edit config -->
        <div v-show="page === 'edit'" class="mt-0 w-full overflow-scroll text-start">

          <peer-summary-island v-if="default_peer_conf.name !== undefined
                                     && default_peer_conf.address !== undefined
                                     && default_peer_conf.endpoint !== undefined"
                               :is-new-peer="true"
                               :peer="default_peer_conf"
                               class="my-2 mr-2"
                               @updated-change-sum="onUpdatedPeerSummaryIslandChangeSum"></peer-summary-island>

          <peer-kind-icon-island v-if="default_peer_conf.kind !== undefined
                                     && default_peer_conf.icon !== undefined"
                                 :peer="default_peer_conf"
                                 class="my-2 mr-2"
                                 @updated-change-sum="onUpdatedPeerKindIconIslandChangeSum"></peer-kind-icon-island>

          <dnsmtu-island v-if="default_peer_conf.dns !== undefined
                               && default_peer_conf.mtu !== undefined"
                         :default-dnsmtu="{dns: network.defaults.peer.dns, mtu: network.defaults.peer.mtu}"
                         :peer="default_peer_conf"
                         class="my-2 mr-2"
                         @updated-change-sum="onUpdatedDnsmtuIslandChangeSum"></dnsmtu-island>

          <scripts-island v-if="default_peer_conf.scripts !== undefined"
                          :peer="default_peer_conf"
                          class="my-2 mr-2"
                          @updated-change-sum="onUpdatedScriptsIslandChangeSum"></scripts-island>

          <peer-details-island v-if="default_peer_conf.private_key !== undefined"
                               :api="api"
                               :peer="default_peer_conf"
                               class="my-2 mr-2"
                               @updated-change-sum="onUpdatedPeerDetailsIslandChangeSum"></peer-details-island>

          <connection-islands v-if="network_w_new_peer"
                              :api="api"
                              :is-new-peer="true"
                              :network="network_w_new_peer"
                              :peer-id="peerId"
                              class="my-2 mr-2"
                              @updated-change-sum="onUpdatedConnectionsIslandsChangeSum"></connection-islands>
        </div>

        <!-- view changes -->
        <div v-show="page === 'view-changes'" class="mt-2 w-full overflow-scroll text-start">
        <change-sum :change-sum="change_sum"
                    :network="network"
                    :peer-id="peerId"></change-sum>
      </div>
      </div>

    </custom-dialog>

    <!-- Dialog: Confirm -->
    <custom-dialog v-if="overlayDialogId === 'confirm-changes'"
                   :left-button-click="() => { overlayDialogId = '' }"
                   :left-button-text="'Cancel'"
                   :right-button-classes="['text-white', 'bg-green-600', 'hover:bg-green-700']"
                   :right-button-click="() => { updateConfiguration(); overlayDialogId = ''; page = 'edit'; $emit('update:dialogId', ''); }"
                   :right-button-text="'Do it!'"
                   class="z-20"
                   icon="danger">
      <h3 class="text-lg leading-6 font-medium text-gray-900">
        Confirm adding peer <strong>{{ change_sum.added_peers[peerId].name }}</strong>
      </h3>
      <div class="mt-2 text-sm text-gray-500">
        Are you sure you want to add this new peer?
      </div>

      <div class="flex max-h-[calc(100vh-20rem)] flex-col overflow-y-auto">
        <change-sum :change-sum="change_sum"
                    :network="network"
                    :peer-id="peerId"></change-sum>
      </div>
    </custom-dialog>

  </div>

</template>

<script>
import CustomDialog from "@/components/dialogs/custom-dialog.vue";
import PeerSummaryIsland from "@/components/islands/peer-summary.vue";
import PeerKindIconIsland from "@/components/islands/peer-kind-icon.vue";
import DNSMTUIsland from "@/components/islands/dnsmtu.vue";
import ScriptsIsland from "@/components/islands/scripts.vue";
import PeerDetails from "@/components/islands/peer-details.vue";
import ConnectionIslands from "@/components/islands/connections.vue";
import ChangeSum from "@/components/change-sum.vue";
import WireGuardHelper from "@/js/wg-helper.js";
import DeleteButton from "@/components/ui/buttons/delete.vue";
import CompareButton from "@/components/ui/buttons/compare.vue";
import EditButton from "@/components/ui/buttons/edit.vue";
import ConfButton from "@/components/ui/buttons/conf.vue";
import QrButton from "@/components/ui/buttons/qr.vue";
import DownloadButton from "@/components/ui/buttons/download.vue";

export default {
  name: "peer-config-dialog",
  components: {
    DownloadButton,
    QrButton,
    ConfButton,
    EditButton,
    CompareButton,
    DeleteButton,
    PeerKindIconIsland,
    'custom-dialog': CustomDialog,
    'peer-summary-island': PeerSummaryIsland,
    'dnsmtu-island': DNSMTUIsland,
    'scripts-island': ScriptsIsland,
    'peer-details-island': PeerDetails,
    'connection-islands': ConnectionIslands,
    'change-sum': ChangeSum,
  },
  props: {
    network: {
      type: Object,
      default: {},
    },
    dialogId: {
      type: String,
      default: "",
    },
    version: {
      type: Object,
      default: {},
    },
    api: {
      type: Object,
      default: null,
    }
  },
  emits: ['update:dialogId'],
  data() {
    return {
      page: "",

      peerSummaryIslandChangeSum: null,
      peerKindIconIslandChangeSum: null,
      dnsmtuIslandChangeSum: null,
      scriptsIslandChangeSum: null,
      peerDetailsIslandChangeSum: null,
      connectionIslandsChangeSum: {
        changed_fields: {},
        added_connections: {},
        removed_connections: {},
        errors: {},
      },
      peerId: "",
      default_peer_conf: {},
      peer_id_address_valid_until: "",
      overlayDialogId: '',
    }
  },
  created() {
    this.page = 'edit'

    this.default_peer_conf = JSON.parse(JSON.stringify(this.network.defaults.peer));

    this.default_peer_conf.name = ""
    this.api.get_network_lease_id_address().then(response => {
      this.peerId = response.peer_id;
      this.default_peer_conf.address = response.address;
      this.peer_id_address_valid_until = response.valid_until;
    });

    this.default_peer_conf.private_key = WireGuardHelper.wg_generate_key();
  },
  methods: {
    onUpdatedPeerSummaryIslandChangeSum(data) {
      this.peerSummaryIslandChangeSum = data;
    },
    onUpdatedPeerKindIconIslandChangeSum(data) {
      this.peerKindIconIslandChangeSum = data;
    },
    onUpdatedDnsmtuIslandChangeSum(data) {
      this.dnsmtuIslandChangeSum = data;
    },
    onUpdatedScriptsIslandChangeSum(data) {
      this.scriptsIslandChangeSum = data;
    },
    onUpdatedPeerDetailsIslandChangeSum(data) {
      this.peerDetailsIslandChangeSum = data;
    },
    onUpdatedConnectionsIslandsChangeSum(data) {
      this.connectionIslandsChangeSum = data;
    },
    updateConfiguration() {
      this.api.patch_network_config({
        added_peers: this.change_sum.added_peers,
        added_connections: this.change_sum.added_connections,
      });
    },
  },
  computed: {
    change_sum() {
      const data = {
        errors: {
          peers: {},
          connections: {},
        },
        added_peers: {},
        added_connections: {},
      };

      // check for errors + changed fields for this peer
      data.errors.peers[this.peerId] = {};
      data.added_peers[this.peerId] = JSON.parse(JSON.stringify(this.default_peer_conf));
      for (const island_datum of [this.peerSummaryIslandChangeSum, this.peerKindIconIslandChangeSum, this.dnsmtuIslandChangeSum, this.scriptsIslandChangeSum, this.peerDetailsIslandChangeSum]) {
        if (!island_datum) continue;
        for (const [island_field, island_value] of Object.entries(island_datum.errors)) {
          if (island_field === "scripts" && island_value) {
            data.errors.peers[this.peerId].scripts = {};
            for (const [script_field, script_value] of Object.entries(island_value)) {
              if (script_field) data.errors.peers[this.peerId].scripts[script_field] = script_value;
            }
            if (Object.keys(data.errors.peers[this.peerId].scripts).length === 0) delete data.errors.peers[this.peerId].scripts;
          } else {
            if (island_value) data.errors.peers[this.peerId][island_field] = island_value;
          }
        }

        for (const [island_field, island_value] of Object.entries(island_datum.changed_fields)) {
          if (island_field === "scripts" && island_value) {
            for (const [script_field, script_value] of Object.entries(island_value)) {
              if (script_field) data.added_peers[this.peerId].scripts[script_field] = script_value;
            }
          } else {
            if (island_value) data.added_peers[this.peerId][island_field] = island_value;
          }
        }
      }

      data.added_connections = this.connectionIslandsChangeSum.added_connections;
      data.errors.connections = this.connectionIslandsChangeSum.errors;

      return data;
    },
    network_w_new_peer() {
      if (this.peerId === "") return null;
      const network = JSON.parse(JSON.stringify(this.network));
      network.peers[this.peerId] = this.change_sum.added_peers[this.peerId];
      return network;

    },
    errorDetected() {
      return !!(Object.keys(this.change_sum.errors.peers[this.peerId]).length + Object.keys(this.change_sum.errors.connections).length)
    },
    peer_wg_conf_file() {
      const wg_network = this.network_w_new_peer;
      wg_network.connections = this.change_sum.added_connections;
      return WireGuardHelper.getPeerConfig(wg_network, this.peerId, this.version.full_version);
    },
  },
}
</script>

<style scoped>

</style>