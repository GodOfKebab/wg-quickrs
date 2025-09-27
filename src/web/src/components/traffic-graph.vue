<template>
  <div class="w-full h-16 relative overflow-hidden">
    <div id="traffic-graph-box" class="h-16">
      <canvas id="traffic-graph"></canvas>
    </div>

    <!-- Tx/Rx labels on the right -->
    <div
        class="absolute top-2 right-5 flex flex-col items-end space-y-1 text-sm text-gray-700 bg-white/80 px-1 py-1 rounded">
      <div class="font-semibold"
           style="color: rgba(59,130,246,0.8)">Tx: {{ tx_avg }}
      </div>
      <div class="font-semibold"
           style="color: rgba(34,197,94,0.8)">Rx: {{ rx_avg }}
      </div>
    </div>
  </div>
</template>

<script>
const {Chart, LineElement, LinearScale, LineController, PointElement, Filler} = await import("chart.js");
Chart.register(
    LineElement,
    LinearScale,
    LineController,
    PointElement,
    Filler
);

function formatThroughputBits(bytesPerSec) {
  if (!isFinite(bytesPerSec) || bytesPerSec <= 0) return "0 b/s";

  const factor = 1000; // SI scaling
  let value = bytesPerSec * 8.; // convert to bits/s

  const units = ["b/s", "kb/s", "Mb/s", "Gb/s", "Tb/s"];

  let unitIndex = 0;
  while (value >= factor && unitIndex < units.length - 1) {
    value /= factor;
    unitIndex++;
  }

  // round to 1 decimal, drop trailing .0
  const rounded = Math.round(value * 10) / 10;
  const text = rounded % 1 === 0 ? String(rounded) : rounded.toFixed(1);

  return `${text} ${units[unitIndex]}`;
}

export default {
  name: "traffic-graph",
  props: {
    network: {
      type: Object,
      default: {},
    },
    telemetry: {
      type: Object,
      default: {},
    },
  },
  data() {
    return {
      tx_avg: "? b/s",
      rx_avg: "? b/s",
      box_div_element: null,
      intervalId: null,
      pos: 0,
      chart: null,
    }
  },
  mounted() {
    this.box_div_element = document.getElementById('traffic-graph-box');

    const ctx = document.getElementById('traffic-graph');
    const chartObj = new Chart(ctx, {
      type: "line",
      data: {
        labels: [0],
        datasets: [
          {
            label: "RX",
            borderColor: "rgba(34,197,94,0.2)", // faint green
            backgroundColor: "rgba(34,197,94,0.05)",
            data: [0],
            tension: 0.3,
            fill: "origin",
            pointRadius: 0,
          },
          {
            label: "TX",
            borderColor: "rgba(59,130,246,0.2)", // faint blue
            backgroundColor: "rgba(59,130,246,0.05)",
            data: [0],
            tension: 0.3,
            fill: "origin",
            pointRadius: 0,
          },
        ],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        animation: false,
        plugins: {},
        scales: {x: {type: "linear", display: false}, y: {type: "linear", display: false, beginAtZero: true}},
      },
    });
    Object.seal(chartObj);
    this.chart = chartObj;
  },
  watch: {
    telemetry: {
      handler() {
        if (Object.keys(this.telemetry.data).length < 2) {
          this.tx_avg = "? b/s";
          this.rx_avg = "? b/s";
          return;
        }
        const txs = [];
        const rxs = [];
        const timestamps = [];
        let prev_telem_data = {};

        for (const telem_data of this.telemetry.data) {
          if (Object.keys(prev_telem_data).length === 0) {
            prev_telem_data = telem_data;
            continue;
          }

          let tx = 0,
              rx = 0;
          const ts = (telem_data.timestamp - prev_telem_data.timestamp) / 1000;

          for (const [connection_id, telemetry_details] of Object.entries(telem_data.datum)) {
            if (!prev_telem_data.datum[connection_id]) continue;

            if (connection_id.startsWith(this.network.this_peer)) {
              tx += telemetry_details.transfer_a_to_b - prev_telem_data.datum[connection_id].transfer_a_to_b;
              rx += telemetry_details.transfer_b_to_a - prev_telem_data.datum[connection_id].transfer_b_to_a;
            } else if (connection_id.endsWith(this.network.this_peer)) {
              tx += telemetry_details.transfer_b_to_a - prev_telem_data.datum[connection_id].transfer_b_to_a;
              rx += telemetry_details.transfer_a_to_b - prev_telem_data.datum[connection_id].transfer_a_to_b;
            }
          }

          txs.push(tx / ts);
          rxs.push(rx / ts);
          timestamps.push(telem_data.timestamp);
          prev_telem_data = telem_data;

        }
        this.tx_avg = formatThroughputBits(txs.at(-1));
        this.rx_avg = formatThroughputBits(rxs.at(-1));

        if (Object.keys(this.telemetry.data).length < 2) return;
        let latest_timestamp_dt = timestamps.at(-1) - timestamps.at(-2);
        while (txs.length < this.telemetry.max_len - 1) {
          txs.unshift(0);
          rxs.unshift(0);
          timestamps.unshift(timestamps[0] - latest_timestamp_dt);
        }

        // Update the chart directly
        this.chart.data.labels = timestamps;
        this.chart.options.scales.x.min = timestamps[0];
        this.chart.options.scales.x.max = timestamps.at(-1);
        this.chart.data.datasets[0].data = rxs;
        this.chart.data.datasets[1].data = txs;
        let ratio_new_data = latest_timestamp_dt / (timestamps.at(-1) - timestamps[0]);
        // console.log(latest_timestamp_dt);
        this.resetMove(ratio_new_data)
        this.chart.update();
        this.startMove(ratio_new_data);
      },
      deep: true
    }
  },
  methods: {
    startMove(ratio) {
      if (this.intervalId) return; // prevent multiple timers
      const duration = 900; // glitches at 1000 ms interval
      const refresh_interval = 20.;
      this.intervalId = setInterval(() => {
        if (this.box_div_element) {
          this.pos += 1;
          const total_margin = ratio * this.box_div_element.getBoundingClientRect().width
          this.box_div_element.style.marginLeft = `${Math.round(-2 * total_margin)}px`;
          const total_transform = total_margin * (1 - this.pos / (duration / refresh_interval));
          this.box_div_element.style.transform = `translateX(${Math.round(total_transform)}px)`;
        }
      }, refresh_interval); // every interval milliseconds
    },
    resetMove(ratio) {
      if (this.intervalId) {
        clearInterval(this.intervalId);
        this.intervalId = null;
      }
      this.pos = 0;
      if (this.box_div_element) {
        const total_margin = ratio * this.box_div_element.getBoundingClientRect().width
        this.box_div_element.style.marginLeft = `${Math.round(-2 * total_margin)}px`;
        this.box_div_element.style.transform = `translateX(${Math.round(total_margin)}px)`;
      }
    }
  }
}
</script>
