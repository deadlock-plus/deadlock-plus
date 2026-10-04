import { t } from "$lib/core/i18n.svelte";

const OFL = "SIL Open Font License 1.1";
const OFL_URL = "https://openfontlicense.org/open-font-license-official-text/";

export type LicenseTone = "open" | "restricted";

export interface LicenseEntry {
    name: string;
    kind: string;
    owner: string;
    notice: string;
    license: string;
    tone: LicenseTone;
    terms: string;
    url?: string;
}

export const APP_LICENSE = {
    name: "Deadlock+",
    spdx: "GPL-3.0-or-later",
    get summary() {
        return t("settings.licenses.app_summary");
    },
    get warranty() {
        return t("settings.licenses.app_warranty");
    },
    url: "https://www.gnu.org/licenses/gpl-3.0.html",
};

export const LICENSES: LicenseEntry[] = [
    {
        name: "Retail Demo",
        get kind() {
            return t("settings.licenses.kind_font");
        },
        owner: "OH no Type Company, LLC (James Edmondson)",
        notice: "(c) Copyright OH no Type Company, LLC. 2021. All rights reserved. Ohno Humanist is a trademark of OH no Type Company, LLC.",
        license: "Proprietary",
        tone: "restricted",
        get terms() {
            return t("settings.licenses.terms_not_open_source");
        },
        url: "http://ohnotype.co/license",
    },
    {
        name: "Valve Oracle",
        get kind() {
            return t("settings.licenses.kind_font");
        },
        owner: "Valve Corporation (YouWorkForThem)",
        notice: "Copyright 2025 Valve Corporation.",
        license: "Proprietary",
        tone: "restricted",
        get terms() {
            return t("settings.licenses.terms_valve_oracle");
        },
    },
    {
        name: "Valve Pulp",
        get kind() {
            return t("settings.licenses.kind_font");
        },
        owner: "Valve Corporation (YouWorkForThem / Nicolas Massi)",
        notice: "Copyright 2025 Valve Corp. All Rights Reserved. Valve Pulp is a trademark of Valve Corporation.",
        license: "Fan art use",
        tone: "restricted",
        get terms() {
            return t("settings.licenses.terms_valve_pulp");
        },
    },
    {
        name: "Atkinson Hyperlegible",
        get kind() {
            return t("settings.licenses.kind_font");
        },
        owner: "Braille Institute of America, Inc.",
        notice: "Copyright 2020 Braille Institute of America, Inc.",
        license: "OFL 1.1",
        tone: "open",
        terms: OFL,
        url: OFL_URL,
    },
    {
        name: "Limelight",
        get kind() {
            return t("settings.licenses.kind_font");
        },
        owner: "Sorkin Type Co",
        notice: "Copyright (c) 2010 by Sorkin Type Co (eben@eyebytes.com) with Reserved Font Name Limelight.",
        license: "OFL 1.1",
        tone: "open",
        terms: OFL,
        url: OFL_URL,
    },
    {
        name: "Twemoji (flag artwork)",
        get kind() {
            return t("settings.licenses.kind_artwork");
        },
        owner: "Twitter, Inc. and other contributors; packaged by Samuel Kopp as @twemoji/svg",
        notice: "Graphics Copyright 2020 Twitter, Inc and other contributors.",
        license: "CC-BY 4.0",
        tone: "open",
        get terms() {
            return t("settings.licenses.terms_twemoji");
        },
        url: "https://creativecommons.org/licenses/by/4.0/",
    },
];

export function disclaimer(): string {
    return t("settings.licenses.disclaimer");
}
