<script>
  import { toast } from "@zerodevx/svelte-toast";
  import { onMount, onDestroy } from "svelte";
  import { createEventDispatcher } from "svelte";
  import { tick } from "svelte";

  const dispatch = createEventDispatcher();

  export let activeIndex = 0;
  let childNodes = [];
  let lastJumpUp = [];
  export let can_expand = (path, do_expand) => false;
  export let can_contract = (path, do_expand) => false;
  export let handle_shift_up = async () => false;
  export let handle_shift_down = async () => false;

  let container;

  export let scroll_to_active = () => {
    childNodes.forEach((node, i) => {
      if (i === activeIndex) {
        // node.classList.add("chosen");
        if (true) {
          node.scrollIntoView({
            behavior: "smooth",
            block: "center",
          });
        }
      }
    });
  };

  function setActiveIndex(index, scroll = true) {
    if (activeIndex !== index && index != -1) {
      const activeChild = childNodes[index];
      const dataPath = activeChild.dataset.path;
      dispatch("itemChanged", { path: dataPath });
    }
    activeIndex = index;
    scroll_to_active();
  }

  async function handleKeyDown(event) {
    switch (event.key) {
      case "ArrowUp":
        event.preventDefault();
        event.stopPropagation();
        if (event.ctrlKey) {
          console.log("shift up");
          if (!(await handle_shift_up())) {
            return;
          }
        }
        let new_index;
        if (event.shiftKey) {
          //todo: skip to next node with same or shorter prefix length

          const last = childNodes[activeIndex];
          const dataPath = last.dataset.path;
          const prefix_length = dataPath.length;
          new_index = Math.max(activeIndex - 1, 0);
          while (
            new_index > 0 &&
            childNodes[new_index].dataset.path.length > prefix_length
          ) {
            new_index -= 1;
          }
        } else {
          new_index = Math.max(activeIndex - 1, 0);
        }
        setActiveIndex(new_index);
        break;
      case "ArrowLeft":
        {
          const last = childNodes[activeIndex];
          const dataPath = last.dataset.path;
          if (can_contract(dataPath, true)) {
          } else {
            console.log(dataPath);
            //remove last character
            let prefix = dataPath.substring(0, dataPath.length - 1);
            let new_index = Math.max(0, activeIndex - 1);
            while (
              new_index > 0 &&
              childNodes[new_index].dataset.path != prefix
            ) {
              new_index -= 1;
            }
            if (lastJumpUp.length > 0) {
              if (!lastJumpUp[lastJumpUp.length - 1].startsWith(dataPath)) {
                lastJumpUp = [];
              }
            }
            lastJumpUp.push(dataPath);
            console.log(lastJumpUp);
            setActiveIndex(new_index);
            event.stopPropagation();
            event.preventDefault();
          }
        }
        break;
      case "ArrowRight":
        {
          let new_index = Math.min(activeIndex + 1, childNodes.length - 1);
          let popped = false;
          if (lastJumpUp.length > 0) {
            let target_path = lastJumpUp.pop();
            const last = childNodes[activeIndex];
            const data_path = last.dataset.path;
            if (can_expand(data_path, true)) {
              console.log("did expand");
              lastJumpUp.push(target_path);
              window.setTimeout(() => {
                handleKeyDown(event);
              }, 10);

              event.stopPropagation();
              event.preventDefault();
              return;
            } else {
              new_index = Math.min(activeIndex + 1, childNodes.length - 1);
              if (target_path.startsWith(data_path)) {
                console.log(
                  "had last",
                  target_path,
                  data_path,
                  childNodes[new_index].dataset.path
                );
                popped = true;
                while (
                  new_index < childNodes.length - 1 &&
                  childNodes[new_index].dataset.path < target_path
                ) {
                  new_index += 1;
                }
              } else {
              }
            }
          }
          if (
            !popped &&
            can_expand(childNodes[activeIndex].dataset.path, true)
          ) {
          }
          setActiveIndex(new_index);
          event.stopPropagation();
          event.preventDefault();
        }
        break;

      case "ArrowDown":
        event.stopPropagation();
        event.preventDefault();
        if (event.ctrlKey) {
          if (!(await handle_shift_down())) {
            return;
          }
        }
        if (event.shiftKey) {
          //todo: refactor into siblings_flat or such...
          const last = childNodes[activeIndex];
          const dataPath = last.dataset.path;
          let new_index = Math.min(activeIndex + 1, childNodes.length - 1);
          while (
            new_index < childNodes.length - 1 &&
            childNodes[new_index].dataset.path.startsWith(dataPath)
          ) {
            new_index += 1;
          }
          setActiveIndex(new_index);
        } else {
          setActiveIndex(Math.min(activeIndex + 1, childNodes.length - 1));
        }
        break;
      case "PageUp":
        setActiveIndex(Math.max(activeIndex - 10, 0));
        event.stopPropagation();
        event.preventDefault();
        break;
      case "PageDown":
        setActiveIndex(Math.min(activeIndex + 10, childNodes.length - 1));
        event.preventDefault();
        event.stopPropagation();
        break;
      case "Home":
        setActiveIndex(0);
        event.preventDefault();
        event.stopPropagation();
        break;
      case "End":
        setActiveIndex(childNodes.length - 1);
        event.preventDefault();
        event.stopPropagation();
        break;
      case "Enter":
        if (!event.ctrlKey) {
          const activeChild = childNodes[activeIndex];
          const dataPath = activeChild.dataset.path;
          dispatch("itemSelected", { path: dataPath });
          event.stopPropagation();
        }
        break;
      default:
        break;
    }
  }

  function handleKeyUp(event) {
    switch (event.key) {
      case "ArrowUp":
      case "ArrowDown":
      case "ArrowRight":
      case "ArrowLeft":
      case "PageUp":
      case "PageDown":
      case "Home":
      case "Enter":
      case "End":
        event.preventDefault();
        event.stopPropagation();
        break;
      default:
        break;
    }
  }

  function handleChildClick(event) {
    let clickedChild = event.target;
    if (event == null) {
      return;
    }
    while (
      clickedChild.parentNode != null &&
      clickedChild.parentNode !== container
    ) {
      clickedChild = clickedChild.parentNode;
    }
    const clickedIndex = childNodes.indexOf(clickedChild);
    console.log("clicked index", clickedIndex);
    setActiveIndex(clickedIndex, false);
  }

  function handleChildDoubleClick(event) {
    const dataPath = childNodes[activeIndex].dataset.path;
    dispatch("itemExpand", { path: dataPath });
  }

  let observer;

  onMount(() => {
    childNodes = Array.from(container.children);
    setActiveIndex(activeIndex);

    observer = new MutationObserver((mutationsList) => {
      childNodes = Array.from(container.children);
      setActiveIndex(activeIndex);
    });

    observer.observe(container, {
      childList: true,
    });
  });

  onDestroy(() => {
    observer.disconnect();
  });

  function handle_on_focus(ev) {
    /*toast.push("focus");
    window.setTimeout(() => {
      scroll_to_active();
    }, 1);
	*/
  }
</script>

<table
  tabindex="0"
  on:keydown={handleKeyDown}
  on:keyup={handleKeyUp}
  on:click={handleChildClick}
  on:dblclick={handleChildDoubleClick}
  bind:this={container}
  autofocus
  on:focus={handle_on_focus}
>
  <slot />
</table>

<style>
  :global(.chosen) {
    background-color: #bfbfff;
  }
</style>
