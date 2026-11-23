<script lang="ts">
  import CalendarIcon from '@lucide/svelte/icons/calendar';
  import {
    type DateValue,
    DateFormatter,
    getLocalTimeZone,
    parseDate,
  } from '@internationalized/date';
  import { cn } from '$lib/utils.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Calendar } from '$lib/components/ui/calendar/index.js';
  import * as Popover from '$lib/components/ui/popover/index.js';

  interface Props {
    value?: string;
    id?: string;
    placeholder?: string;
    class?: string;
    onchange?: (v: string) => void;
  }

  let {
    value = $bindable(''),
    id,
    placeholder = 'Pilih tanggal',
    class: className,
    onchange,
  }: Props = $props();

  const df = new DateFormatter('id-ID', { dateStyle: 'long' });

  let calendarValue = $derived.by<DateValue | undefined>(() => {
    if (!value) return undefined;
    try { return parseDate(value); } catch { return undefined; }
  });

  function onCalendarSelect(v: DateValue | undefined) {
    if (!v) return;
    value = v.toString();
    onchange?.(value);
    open = false;
  }

  let open = $state(false);
</script>

<!-- Wrapper w-full ensures the trigger fills its container regardless of Popover internals -->
<div class={cn('w-full', className)}>
  <Popover.Root bind:open>
    <Popover.Trigger class="w-full">
      {#snippet child({ props })}
        <Button
          variant="outline"
          {id}
          size="default"
          class={cn(
            'w-full h-8 justify-start gap-2 text-left text-sm font-normal px-2.5',
            !calendarValue && 'text-muted-foreground'
          )}
          {...props}
        >
          <CalendarIcon class="size-4 shrink-0 text-muted-foreground" />
          <span class="truncate text-left flex-1">
            {calendarValue
              ? df.format(calendarValue.toDate(getLocalTimeZone()))
              : placeholder}
          </span>
        </Button>
      {/snippet}
    </Popover.Trigger>
    <Popover.Content class="w-auto p-0" align="start">
      <Calendar
        type="single"
        value={calendarValue}
        onValueChange={onCalendarSelect}
        locale="id-ID"
        captionLayout="dropdown"
        initialFocus
      />
    </Popover.Content>
  </Popover.Root>
</div>
