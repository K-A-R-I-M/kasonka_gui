<script lang="ts">
  import Card from "$lib/Card.svelte";
  import type { Audio } from "$lib/types";

  export let listAudios: Array<Audio>;
</script>

<div class="list">
  {#each listAudios as audio}
    <Card title={audio.title} duration={audio.duration} />
  {/each}
</div>
