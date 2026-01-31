// ComboEditor - Dialog for creating/editing combos
import { useEffect, useRef, useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import * as Dialog from "@radix-ui/react-dialog";
import * as Select from "@radix-ui/react-select";
import { X, ChevronDown, Check } from "lucide-react";
import { useGroupStore } from "../../stores/groupStore";
import { createComboSchema } from "../../lib/schemas";
import { InsertVariableMenu } from "./InsertVariableMenu";
import { SnippetEditor } from "./SnippetEditor";
import type { Combo, CreateComboInput } from "../../lib/types";
import type { z } from "zod";

interface ComboEditorProps {
  open: boolean;
  combo?: Combo;
  onSave: (data: CreateComboInput) => Promise<void> | void;
  onCancel: () => void;
}

type ComboFormData = z.infer<typeof createComboSchema>;

export function ComboEditor({ open, combo, onSave, onCancel }: ComboEditorProps) {
  const { groups } = useGroupStore();
  const snippetRef = useRef<HTMLTextAreaElement>(null);
  const [submitting, setSubmitting] = useState(false);

  const {
    register,
    handleSubmit,
    formState: { errors },
    reset,
    setValue,
    watch,
  } = useForm<ComboFormData>({
    resolver: zodResolver(createComboSchema),
    defaultValues: {
      name: "",
      description: "",
      keyword: "",
      snippet: "",
      groupId: groups[0]?.id || "",
      matchingMode: "strict",
      caseSensitive: false,
      enabled: true,
    },
  });

  // Reset form when combo or open changes
  useEffect(() => {
    if (open) {
      if (combo) {
        reset({
          name: combo.name,
          description: combo.description,
          keyword: combo.keyword,
          snippet: combo.snippet,
          groupId: combo.groupId,
          matchingMode: combo.matchingMode,
          caseSensitive: combo.caseSensitive,
          enabled: combo.enabled,
        });
      } else {
        reset({
          name: "",
          description: "",
          keyword: "",
          snippet: "",
          groupId: groups[0]?.id || "",
          matchingMode: "strict",
          caseSensitive: false,
          enabled: true,
        });
      }
    }
  }, [open, combo, reset, groups]);

  const onSubmit = async (data: ComboFormData) => {
    setSubmitting(true);
    try {
      await onSave(data);
    } finally {
      setSubmitting(false);
    }
  };

  // Insert variable at cursor position in the snippet editor
  const insertVariable = (variable: string) => {
    if (!snippetRef.current) return;

    const textarea = snippetRef.current;
    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const currentValue = watch("snippet");
    const newValue =
      currentValue.substring(0, start) + variable + currentValue.substring(end);

    setValue("snippet", newValue);

    // Set cursor after inserted variable
    setTimeout(() => {
      textarea.focus();
      textarea.setSelectionRange(start + variable.length, start + variable.length);
    }, 0);
  };

  if (!open) return null;

  return (
    <Dialog.Root open={open} onOpenChange={(isOpen) => !isOpen && onCancel()}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/50" />
        <Dialog.Content className="fixed top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 bg-white rounded-lg shadow-xl p-6 w-full max-w-2xl max-h-[90vh] overflow-y-auto">
          <div className="flex items-center justify-between mb-4">
            <Dialog.Title className="text-2xl font-bold">
              {combo ? "Edit Combo" : "Create Combo"}
            </Dialog.Title>
            <button
              onClick={onCancel}
              className="p-1 hover:bg-gray-100 rounded"
              aria-label="Close"
            >
              <X className="w-5 h-5" />
            </button>
          </div>
          <Dialog.Description className="text-gray-600 mb-4">
            {combo
              ? "Update the combo details below"
              : "Create a new text expansion combo"}
          </Dialog.Description>

        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
          {/* Name */}
          <div>
            <label htmlFor="name" className="block text-sm font-medium mb-1">
              Name *
            </label>
            <input
              id="name"
              type="text"
              {...register("name")}
              className="w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="e.g., Email Signature"
            />
            {errors.name && (
              <p className="text-red-500 text-sm mt-1">{errors.name.message}</p>
            )}
          </div>

          {/* Description */}
          <div>
            <label htmlFor="description" className="block text-sm font-medium mb-1">
              Description
            </label>
            <input
              id="description"
              type="text"
              {...register("description")}
              className="w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="Optional description"
            />
          </div>

          {/* Keyword */}
          <div>
            <label htmlFor="keyword" className="block text-sm font-medium mb-1">
              Keyword *
            </label>
            <input
              id="keyword"
              type="text"
              {...register("keyword")}
              className="w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono"
              placeholder="e.g., sig (no spaces)"
            />
            {errors.keyword && (
              <p className="text-red-500 text-sm mt-1">{errors.keyword.message}</p>
            )}
          </div>

          {/* Snippet */}
          <div>
            <div className="flex items-center justify-between mb-1">
              <label htmlFor="snippet" className="block text-sm font-medium">
                Snippet *
              </label>
              <InsertVariableMenu onInsert={insertVariable} />
            </div>
            <SnippetEditor
              id="snippet"
              ref={snippetRef}
              value={watch("snippet")}
              onChange={(value) => setValue("snippet", value)}
              placeholder="The text to expand..."
              rows={6}
              className="w-full"
            />
            {errors.snippet && (
              <p className="text-red-500 text-sm mt-1">{errors.snippet.message}</p>
            )}
          </div>

          {/* Group */}
          <div>
            <label htmlFor="group" className="block text-sm font-medium mb-1">
              Group *
            </label>
            <Select.Root
              value={watch("groupId")}
              onValueChange={(value) => setValue("groupId", value)}
            >
              <Select.Trigger
                id="group"
                className="w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 flex items-center justify-between"
                aria-label="Group"
              >
                <Select.Value />
                <Select.Icon>
                  <ChevronDown className="w-4 h-4" />
                </Select.Icon>
              </Select.Trigger>
              <Select.Portal>
                <Select.Content className="bg-white border rounded-lg shadow-lg">
                  <Select.Viewport className="p-1">
                    {groups.map((group) => (
                      <Select.Item
                        key={group.id}
                        value={group.id}
                        className="px-3 py-2 hover:bg-gray-100 rounded cursor-pointer flex items-center justify-between"
                      >
                        <Select.ItemText>{group.name}</Select.ItemText>
                        <Select.ItemIndicator>
                          <Check className="w-4 h-4" />
                        </Select.ItemIndicator>
                      </Select.Item>
                    ))}
                  </Select.Viewport>
                </Select.Content>
              </Select.Portal>
            </Select.Root>
          </div>

          {/* Matching Mode */}
          <div>
            <label className="block text-sm font-medium mb-2">Matching Mode *</label>
            <div className="space-y-2">
              <label className="flex items-center gap-2">
                <input
                  type="radio"
                  {...register("matchingMode")}
                  value="strict"
                  className="w-4 h-4"
                />
                <span>Strict (match after word boundary)</span>
              </label>
              <label className="flex items-center gap-2">
                <input
                  type="radio"
                  {...register("matchingMode")}
                  value="loose"
                  className="w-4 h-4"
                />
                <span>Loose (match anywhere)</span>
              </label>
            </div>
          </div>

          {/* Options */}
          <div className="space-y-2">
            <label className="flex items-center gap-2">
              <input
                type="checkbox"
                {...register("caseSensitive")}
                className="w-4 h-4"
              />
              <span className="text-sm">Case Sensitive</span>
            </label>
            <label className="flex items-center gap-2">
              <input type="checkbox" {...register("enabled")} className="w-4 h-4" />
              <span className="text-sm">Enabled</span>
            </label>
          </div>

          {/* Actions */}
          <div className="flex justify-end gap-2 pt-4 border-t">
            <button
              type="button"
              onClick={onCancel}
              className="px-4 py-2 border rounded-lg hover:bg-gray-50"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={submitting}
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-400"
            >
              {submitting ? "Saving..." : "Save"}
            </button>
          </div>
        </form>
      </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
