<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { getContext, onMount, onDestroy } from "svelte";
  import type { AudioKa } from "$lib/types";
  const { get_cta_from_back, time_to_str, str_to_time }: any =
    getContext("cta");

  export let play: boolean;
  export let current_audio: AudioKa;
  let current_audio_inside: AudioKa = {
    title: "",
    duration: "",
    duration_nb: 0,
  };
  let current_title_audio: string = "Audio";
  let current_duration_audio_str: string = "00:00:00";
  let current_duration_audio: number = 0;
  let current_time_audio_str: string = "00:00:00";
  let current_time_audio: number = 0;
  let intervalId: number;
  let input_range: HTMLInputElement;

  async function resume() {
    play = await invoke("resume_pause");
  }

  async function next() {
    await invoke("next");
    current_audio = await get_cta_from_back();
  }

  async function previous() {
    await invoke("previous");
    current_audio = await get_cta_from_back();
  }

  async function handleAudioTime() {
    invoke("set_audio_time", { audioTime: current_time_audio });
  }

  async function get_cta_inside() {
    if (current_audio.title !== current_audio_inside.title) {
      current_time_audio = 0;
      current_time_audio_str = "00:00:00";
    } else {
      if (play) {
        current_time_audio += 1;

        current_time_audio_str = time_to_str(current_time_audio);
      }
    }
    current_audio_inside = current_audio;
    current_title_audio = current_audio_inside.title;
    current_duration_audio_str = current_audio_inside.duration;
    current_duration_audio = current_audio_inside.duration_nb;

    input_range.max = str_to_time(current_duration_audio_str);
  }

  // Start checking the API when the component is mounted
  onMount(() => {
    get_cta_inside(); // Initial fetch
    intervalId = setInterval(get_cta_inside, 1000); // Fetch data every 5 seconds (adjust as needed)
  });

  // Stop checking the API when the component is destroyed
  onDestroy(() => {
    clearInterval(intervalId);
  });

  if (typeof window !== "undefined") {
    $: get_cta_inside();
  }
</script>

<div
  class="flex flex-col fixed bottom-0 left-0 right-0 justify-around bg-gray-900 text-white text-base pt-2 py-4 px-4 h-28"
>
  <div class="flex space-x-2 justify-center items-center w-full pb-2">
    <p>{current_time_audio_str}</p>
    <input
      class=" w-4/5"
      type="range"
      min="0"
      bind:this={input_range}
      step="1"
      bind:value={current_time_audio}
      on:change={handleAudioTime}
      disabled
    />
    <p>{current_duration_audio_str}</p>
  </div>
  <div class="flex">
    <div class="container flex flex-col m-auto">
      <div class="flex items-center justify-start">
        <p>{current_title_audio}</p>
      </div>
    </div>
    <div class="flex items-center justify-center space-x-4">
      <button
        on:click|preventDefault={previous}
        class="text-white hover:text-gray-400"
      >
        Previous
      </button>
      <button
        on:click|preventDefault={resume}
        class="bg-green-500 hover:bg-green-600 text-white font-semibold py-2 px-4 rounded-full"
      >
        {#if play}
          Pause
        {:else}
          Play
        {/if}
      </button>
      <button
        on:click|preventDefault={next}
        class="text-white hover:text-gray-400"
      >
        Next
      </button>
    </div>
    <nav class="container my-auto text-center">
      <ul class="flex justify-end space-x-4">
        <li><a href="/" class="hover:text-gray-400">Home</a></li>
        <li><a href="/playlist" class="hover:text-gray-400">Playlist</a></li>
        <li><a href="/credits" class="hover:text-gray-400">Credits</a></li>
        <li><a href="/settings" class="hover:text-gray-400">Settings</a></li>
      </ul>
    </nav>
  </div>
</div>
