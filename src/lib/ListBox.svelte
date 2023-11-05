<!-- ListBox.svelte -->
<script lang="ts">
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();

  export let options: string[];
  export let selectedOption: string;
  let isOpen = false;

  function toggleListbox() {
    isOpen = !isOpen;
  }

  function selectOption(option: string) {
    selectedOption = option;
    isOpen = false;
    dispatch("select", { data: option });
  }
</script>

<div class="relative inline-block text-left">
  <div class="relative">
    <button
      type="button"
      class="bg-gray-900 text-white text-sm p-4 flex justify-center items-center w-100 min-w-max"
      id="listbox"
      aria-haspopup="listbox"
      on:click={toggleListbox}
    >
      {#if selectedOption}
        {selectedOption}
      {:else}
        Select an option
      {/if}
    </button>
  </div>

  {#if isOpen}
    <div
      class=" absolute left-0 right-0 mt-2 w-100 bg-gray-900 text-white z-50"
      role="listbox"
      aria-labelledby="listbox"
      tabindex="-1"
    >
      {#each options as option (option)}
        <div
          role="option"
          on:click={() => selectOption(option)}
          on:keyup
          on:keydown
          class="cursor-pointer select-none relative p-2"
          tabindex="-1"
          id={option}
          aria-selected={option === selectedOption}
        >
          <span
            class={`block text-sm truncate ${
              option === selectedOption ? "font-bold" : ""
            }`}>{option}</span
          >
        </div>
      {/each}
    </div>
  {/if}
</div>
