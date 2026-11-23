<script lang="ts">
	import { Select as SelectPrimitive } from "bits-ui";

	let {
		open = $bindable(false),
		value = $bindable(),
		...restProps
	}: Omit<Extract<SelectPrimitive.RootProps, { type: 'single' }>, 'type'> = $props();
</script>

<SelectPrimitive.Root bind:open bind:value type="single" {...restProps} />
