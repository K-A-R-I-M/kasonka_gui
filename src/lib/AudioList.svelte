<script lang="ts">
  import CardList from "$lib/CardList.svelte";
  import Button from "$lib/Button.svelte";
  import { invoke } from "@tauri-apps/api/tauri";
  import type { Item } from "$lib/types";
  import { onDestroy, onMount } from "svelte";

  let listItems: Item[] = [];

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
    intervalId = setInterval(updateList, 1500); // Fetch data every 5 seconds (adjust as needed)
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
