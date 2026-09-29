const OFL = "SIL Open Font License 1.1";
const OFL_URL = "https://openfontlicense.org/open-font-license-official-text/";

export type LicenseTone = "open" | "restricted";

export interface LicenseEntry {
    name: string;
    kind: "Font" | "Artwork";
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
    summary:
        "Deadlock+ is free software. You can use, share and change it under the GNU General Public License, version 3 or any later version.",
    warranty: "It comes with no warranty.",
    url: "https://www.gnu.org/licenses/gpl-3.0.html",
};

export const LICENSES: LicenseEntry[] = [
    {
        name: "Retail Demo",
        kind: "Font",
        owner: "OH no Type Company, LLC (James Edmondson)",
        notice: "(c) Copyright OH no Type Company, LLC. 2021. All rights reserved. Ohno Humanist is a trademark of OH no Type Company, LLC.",
        license: "Proprietary",
        tone: "restricted",
        terms: "Not open source.",
        url: "http://ohnotype.co/license",
    },
    {
        name: "Valve Oracle",
        kind: "Font",
        owner: "Valve Corporation (YouWorkForThem)",
        notice: "Copyright 2025 Valve Corporation.",
        license: "Proprietary",
        tone: "restricted",
        terms: "The font file points to its licence metadata for details.",
    },
    {
        name: "Valve Pulp",
        kind: "Font",
        owner: "Valve Corporation (YouWorkForThem / Nicolas Massi)",
        notice: "Copyright 2025 Valve Corp. All Rights Reserved. Valve Pulp is a trademark of Valve Corporation.",
        license: "Fan art use",
        tone: "restricted",
        terms: "Like other Valve game content, may be used to create Fan Art, as described in the Steam Subscriber Agreement.",
    },
    {
        name: "Atkinson Hyperlegible",
        kind: "Font",
        owner: "Braille Institute of America, Inc.",
        notice: "Copyright 2020 Braille Institute of America, Inc.",
        license: "OFL 1.1",
        tone: "open",
        terms: OFL,
        url: OFL_URL,
    },
    {
        name: "Limelight",
        kind: "Font",
        owner: "Sorkin Type Co",
        notice: "Copyright (c) 2010 by Sorkin Type Co (eben@eyebytes.com) with Reserved Font Name Limelight.",
        license: "OFL 1.1",
        tone: "open",
        terms: OFL,
        url: OFL_URL,
    },
    {
        name: "Twemoji (flag artwork)",
        kind: "Artwork",
        owner: "Twitter, Inc. and other contributors; packaged by Samuel Kopp as @twemoji/svg",
        notice: "Graphics Copyright 2020 Twitter, Inc and other contributors.",
        license: "CC-BY 4.0",
        tone: "open",
        terms: "The npm package code is MIT.",
        url: "https://creativecommons.org/licenses/by/4.0/",
    },
];

export const DISCLAIMER =
    'Deadlock+ is an unofficial fan tool, not made by, affiliated with or endorsed by Valve Corporation. "Deadlock" and "Steam" are trademarks of Valve Corporation. The Deadlock fonts remain their owners\' property and can be removed on request.';
