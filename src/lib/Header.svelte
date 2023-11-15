<script lang="ts">
  import Button from "$lib/Button.svelte";
  import { invoke } from "@tauri-apps/api/tauri";
  import type { AudioKa } from "$lib/types";
  import { getContext } from "svelte";

  const { update_cta, update_play, notify, get_cta_from_back }: any =
    getContext("cta");

  let title: string = "";
  let btn_text = "+";
  export let current_audio: AudioKa;

  async function get_cta_inside(): Promise<AudioKa> {
    current_audio = await get_cta_from_back();
    return current_audio;
  }

  async function addaudio() {
    notify("Start downloading", '"' + title + '" will be downloaded');
    let audio_processed: [string, boolean] = await invoke("add_audio", {
      titleAudio: title,
    });

    let audio_title_processed: AudioKa = {
      title: audio_processed[0],
      duration: "00:00",
      duration_nb: 0,
    };
    let played: boolean = audio_processed[1];

    if (played) {
      update_cta(audio_title_processed);
      update_play(true);
    }
    title = "";
  }
</script>

<div
  class="fixed top-0 left-0 right-0 flex justify-center items-center bg-gray-900 text-white py-1"
>
  <form class="flex" on:submit|preventDefault={addaudio}>
    <input
      id="add-input-title"
      class="p-0 px-2 w-72 bg-black text-left placeholder:text-center"
      placeholder="Audio Title"
      bind:value={title}
    />
    <Button value={btn_text} />
  </form>
</div>
