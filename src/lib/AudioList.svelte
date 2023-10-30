<script lang="ts">
  import CardList from "$lib/CardList.svelte";
  import Button from "$lib/Button.svelte";
  import Notification from "$lib/Notification.svelte";
  import { invoke } from "@tauri-apps/api/tauri";
  import type { Item } from "$lib/types";
  import { getContext, onDestroy, onMount } from "svelte";
  const { update_cta, update_play, notify }: any = getContext("cta");

  let listItems: Item[] = [];

  let title = "";

  let btn_text = "+";

  async function loadList() {
    let audio_title_processed: Array<string> = await invoke("get_list_audio");
    if (audio_title_processed.length >= 0) {
      audio_title_processed.forEach(function (audio_title) {
        const newItem: Item = {
          title: audio_title,
          duration: "3:00",
        };

        listItems = [...listItems, newItem];
      });
    }
  }

  async function addaudio() {
    notify(title);
    let audio_processed: [string, boolean] = await invoke("add_audio", {
      titleAudio: title,
    });

    let audio_title_processed: string = audio_processed[0];
    let played: boolean = audio_processed[1];

    const newItem: Item = {
      title: audio_title_processed,
      duration: "3:00",
    };

    listItems = [...listItems, newItem];

    if (played) {
      update_cta(audio_title_processed);
      update_play(true);
    }
    title = "";
  }

  let intervalId: number;

  async function updateList() {
    let audio_title_processed: Array<string> = await invoke("get_list_audio");
    let tmplistItems: Item[] = [];
    if (audio_title_processed.length >= 0) {
      audio_title_processed.forEach(function (audio_title) {
        const newItem: Item = {
          title: audio_title,
          duration: "3:00",
        };

        tmplistItems = [...tmplistItems, newItem];
      });
    }
    listItems = tmplistItems;
  }

  // Start checking the API when the component is mounted
  onMount(() => {
    updateList(); // Initial fetch
    intervalId = setInterval(updateList, 5000); // Fetch data every 5 seconds (adjust as needed)
  });

  // Stop checking the API when the component is destroyed
  onDestroy(() => {
    clearInterval(intervalId);
  });

  if (typeof window !== "undefined") {
    $: loadList();
  }
</script>

<div class="overflow-auto pb-5 mb-5">
  <CardList {listItems} />
</div>
<div class="fixed-bottom h-auto rounded-md">
  <form class="flex" on:submit|preventDefault={addaudio}>
    <input
      id="add-input-title"
      class="text-black p-2 text-center"
      placeholder="Audio Title"
      bind:value={title}
    />
    <Button value={btn_text} />
  </form>
</div>
