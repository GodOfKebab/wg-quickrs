<template>

  <div class="text-sm text-gray-500 whitespace-pre grid grid-cols-2 gap-1">
    <div v-show="changeSum.errors" class="col-span-2 bg-red-100 rounded-md overflow-scroll">
      <strong class="text-gray-600 justify-center rounded-md bg-red-200 p-1">Errors</strong>
      {{ JSON.stringify(changeSum.errors, false, 2) }}
    </div>
    <div v-show="changeSum.changed_fields" class="col-span-2 bg-blue-100 rounded-md overflow-scroll">
      <strong class="text-gray-600 justify-center rounded-md bg-blue-200 p-1">Changed fields</strong>
      {{ JSON.stringify(changeSum.changed_fields, false, 2) }}
    </div>
    <div v-show="changeSum.added_connections" class="col-span-2 bg-green-100 rounded-md overflow-scroll">
      <strong class="text-gray-600 justify-center rounded-md bg-green-200 p-1">Added fields</strong>
      {{ JSON.stringify(changeSum.added_connections, false, 2) }}
    </div>
    <div v-show="changeSum.removed_connections" class="col-span-2 bg-gray-100 rounded-md overflow-scroll">
      <strong class="text-gray-600 justify-center rounded-md bg-gray-200 p-1">Removed fields</strong>
      {{ JSON.stringify(changeSum.removed_connections, false, 2) }}
    </div>
    <div class="bg-gray-100 rounded-md overflow-scroll">
      <strong class="text-gray-600 justify-center rounded-md bg-gray-200 p-1">Old configuration</strong>
      <div class="p-1">{{ JSON.stringify(network, false, 2) }}</div>
    </div>
    <div class="bg-green-100 rounded-md overflow-scroll">
      <strong class="text-gray-600 justify-center rounded-md bg-green-200 p-1">New configuration</strong>
      <div class="p-1">{{ JSON.stringify(new_network, false, 2) }}</div>
    </div>
  </div>

</template>

<script>


export default {
  name: "change-sum",
  props: {
    changeSum: {
      type: Object,
      default: {
        errors: {
          peers: {},
          connections: {},
        },
        changed_fields: {
          peers: {},
          connections: {},
        },
        added_connections: {},
        removed_connections: {}
      },
    },
    peerId: {
      type: String,
      default: "",
    },
    network: {
      type: Object,
      default: {},
    },
  },
  computed: {
    new_network() {
      let new_network = JSON.parse(JSON.stringify(this.network));

      if (!this.changeSum.changed_fields) {
        return new_network;
      }
      for (const peer_field in this.changeSum.changed_fields.peers) {
        if (peer_field === "scripts") {
          for (const script_field in this.changeSum.changed_fields.peers.scripts[peer_field]) {
            new_network.peers[this.peerId].scripts[script_field] = this.changeSum.changed_fields.peers.scripts[script_field];
          }
          continue;
        }
        new_network.peers[this.peerId][peer_field] = this.changeSum.changed_fields.peers[peer_field];
      }

      return new_network
    },
  }
}
</script>

<style scoped>

</style>