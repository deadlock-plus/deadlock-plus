import type { ParamMatcher } from "@sveltejs/kit";
import { CATEGORIES } from "$lib/features/settings/catalog";

export const match: ParamMatcher = (param) => CATEGORIES.some((c) => c.id === param);
