<template>

  <div>
    <!-- Dialog: Settings -->
    <custom-dialog :left-button-click="() => { dialogId = ''; $emit('update:dialogId', dialogId); }"
                   :left-button-text="'Cancel'"
                   :right-button-classes="['enabled:bg-green-700', 'enabled:hover:bg-green-800', 'enabled:focus:outline-none', 'bg-gray-200', 'disabled:hover:bg-gray-200', 'disabled:cursor-not-allowed', 'text-white']"
                   :right-button-click="() => { dialogId = 'confirm-changes'; }"
                   :rightButtonDisabled="peerConfigWindow !== 'file' && errorDetected"
                   class="z-10"
                   right-button-text="Create Peer">

      <!-- title and top bar -->
      <div class="flex flex-col items-center">
        <h3 class="text-lg leading-6 font-medium text-gray-900 inline mb-2 text-start w-full">
          Create a new Peer:
        </h3>
        <span class="order-last w-full flex justify-between px-1">
          <button
              class="align-middle bg-gray-100 disabled:opacity-40 hover:enabled:bg-gray-600 p-1 px-2 rounded transition special-fill"
              disabled
              title="Delete this peer">
            <img alt="Delete" class="h-6" src="../icons/flowbite/trash-bin.svg"/>
          </button>
          <button
              class="align-middle bg-gray-100 disabled:opacity-40 hover:enabled:bg-gray-600 p-1 px-2 rounded transition special-fill"
                  title="See the configuration differences for this peer"
                  @click="peerConfigWindow = 'view-changes'">
            <img alt="Compare Configuration" class="h-6" src="../icons/flowbite/merge-cells.svg"/>
          </button>
          <button
              class="align-middle bg-gray-100 disabled:opacity-40 hover:enabled:bg-gray-600 p-1 px-2 rounded transition special-fill-edit"
              title="Edit the configuration for this peer"
              @click="peerConfigWindow = 'edit'">
            <img alt="Edit Configuration" class="h-6" src="../icons/flowbite/file-pen.svg"/>
          </button>
          <button
              class="align-middle bg-gray-100 disabled:opacity-40 hover:enabled:bg-gray-600 p-1 px-2 rounded transition special-fill"
              disabled
              title="See the configuration file for this peer">
            <img alt="WireGuard Configuration File" class="h-6" src="../icons/flowbite/file-code.svg"/>
          </button>
          <button
              class="align-middle bg-gray-100 disabled:opacity-40 hover:enabled:bg-gray-600 p-1 px-2 rounded transition special-fill"
              disabled
              title="Show QR Code">
            <img alt="QR Code" class="h-6" src="../icons/flowbite/qr-code.svg"/>
          </button>
          <button
              class="align-middle bg-gray-100 disabled:opacity-40 hover:enabled:bg-gray-600 p-1 rounded transition special-fill"
              disabled
              title="Download Configuration">
            <img alt="Download" class="h-6" src="../icons/flowbite/download.svg"/>
          </button>
        </span>
      </div>

      <!-- edit config -->
      <div v-show="peerConfigWindow === 'edit'" class="mt-0 w-full overflow-scroll h-96">

        <peer-summary-island v-if="default_peer_conf.name !== undefined
                                   && default_peer_conf.address !== undefined
                                   && default_peer_conf.endpoint !== undefined"
                             :is-new-peer="true"
                             :peer="default_peer_conf"
                             class="my-2 mr-2"
                             @updated-change-sum="onUpdatedPeerSummaryIslandChangeSum"></peer-summary-island>

        <dnsmtu-island v-if="default_peer_conf.dns !== undefined
                             && default_peer_conf.mtu !== undefined"
                       :default-dnsmtu="{dns: network.defaults.peer.dns, mtu: network.defaults.peer.mtu}"
                       :peer="default_peer_conf"
                       class="my-2 mr-2"
                       @updated-change-sum="onUpdatedDnsmtuIslandChangeSum"></dnsmtu-island>

        <scripts-island v-if="default_peer_conf.scripts !== undefined"
                        :default-scripts="network.defaults.peer.scripts"
                        :peer="default_peer_conf"
                        class="my-2 mr-2"
                        @updated-change-sum="onUpdatedScriptsIslandChangeSum"></scripts-island>

        <peer-details-island v-if="default_peer_conf.private_key !== undefined
                                   && default_peer_conf.public_key !== undefined"
                             :peer="default_peer_conf"
                             class="my-2 mr-2"
                             @updated-change-sum="onUpdatedPeerDetailsIslandChangeSum"></peer-details-island>

        <connection-islands v-if="network_w_new_peer"
                            :is-new-peer="true"
                            :network="network_w_new_peer"
                            :peer-id="peerId"
                            class="my-2 mr-2"
                            @updated-change-sum="onUpdatedConnectionsIslandsChangeSum"></connection-islands>
      </div>

      <!-- view changes -->
      <div v-show="peerConfigWindow === 'view-changes'" class="mt-2 w-full overflow-scroll h-96">
        <change-sum :change-sum="change_sum"
                    :network="network"
                    :peer-id="peerId"></change-sum>
      </div>
    </custom-dialog>

    <!-- Dialog: Confirm -->
    <custom-dialog v-if="dialogId === 'confirm-changes'"
                   :left-button-click="() => { dialogId = ''; $emit('update:dialogId', dialogId); }"
                   :left-button-text="'Cancel'"
                   :right-button-classes="['text-white', 'bg-green-600', 'hover:bg-green-700']"
                   :right-button-click="() => { updateConfiguration(); dialogId = ''; peerConfigWindow = 'edit'; $emit('update:dialogId', dialogId); }"
                   :right-button-text="'Do it!'"
                   class="z-20"
                   icon="danger">
      <h3 class="text-lg leading-6 font-medium text-gray-900">
        Confirm adding peer <strong>{{ change_sum.added_peers[peerId].name }}</strong>
      </h3>
      <div class="mt-2 text-sm text-gray-500">
        Are you sure you want to add this new peer?
      </div>

      <change-sum :change-sum="change_sum"
                  :network="network"
                  :peer-id="peerId"></change-sum>
    </custom-dialog>

  </div>

</template>

<script>
import CustomDialog from "./custom-dialog.vue";
import PeerSummaryIsland from "./islands/peer-summary.vue";
import DNSMTUIsland from "./islands/dnsmtu.vue";
import ScriptsIsland from "./islands/scripts.vue";
import PeerDetails from "./islands/peer-details.vue";
import ConnectionIslands from "./islands/connections.vue";
import ChangeSum from "./change-sum.vue";
import API from "../js/api.js";

export default {
  name: "peer-config-window",
  components: {
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
    }
  },
  emits: ['update:dialogId'],
  data() {
    return {
      peerConfigWindow: "",

      peerSummaryIslandChangeSum: null,
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
      peer_id_address_valid_until: ""
    }
  },
  created() {
    this.peerConfigWindow = 'edit'

    this.default_peer_conf = JSON.parse(JSON.stringify(this.network.defaults.peer));

    this.default_peer_conf.name = ""
    API.get_lease_id_address().then(response => {
      this.peerId = response.peer_id;
      this.default_peer_conf.address = response.address;
      this.peer_id_address_valid_until = response.valid_until;
    });

    API.get_public_private_key().then(response => {
      this.default_peer_conf.public_key = response.public_key;
      this.default_peer_conf.private_key = response.private_key;
    });
  },
  methods: {
    onUpdatedPeerSummaryIslandChangeSum(data) {
      this.peerSummaryIslandChangeSum = data;
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
      API.patch_network_config(this.change_sum);
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
      for (const island_datum of [this.peerSummaryIslandChangeSum, this.dnsmtuIslandChangeSum, this.scriptsIslandChangeSum, this.peerDetailsIslandChangeSum]) {
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
      return Object.keys(this.change_sum.errors.peers[this.peerId]).length
          + Object.keys(this.change_sum.errors.connections).length
    }
  },
}
</script>

<style scoped>

</style>