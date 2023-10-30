<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";

  let volume_value = 0;

  async function handleVolumeChange() {
    invoke("set_volume", { volumeValue: volume_value });
  }

  async function get_volume() {
    volume_value = await invoke("get_volume");
  }

  if (typeof window !== "undefined") {
    $: get_volume();
  }
</script>

<div class="pt-4 px-48 mx-12">
  <div class="flex flex-col justify-center items-center">
    <label for="steps-range" class="block mb-2 text-sm font-medium text-white"
      >General Volume</label
    >

    <label for="steps-range" class="block mb-2 text-sm font-medium text-white"
      >{volume_value}</label
    >
    <input
      id="steps-range"
      type="range"
      min="0"
      max="100"
      bind:value={volume_value}
      step="1"
      on:change={handleVolumeChange}
      class="w-full h-2 rounded-lg accent-blue-700 cursor-pointer"
    />
  </div>
</div>
