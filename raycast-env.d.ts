/// <reference types="@raycast/api">

/* 🚧 🚧 🚧
 * This file is auto-generated from the extension's manifest.
 * Do not modify manually. Instead, update the `package.json` file.
 * 🚧 🚧 🚧 */

/* eslint-disable @typescript-eslint/ban-types */

type ExtensionPreferences = {}

/** Preferences accessible in all the extension's commands */
declare type Preferences = ExtensionPreferences

declare namespace Preferences {
  /** Preferences accessible in the `lookup-ip-or-asn` command */
  export type LookupIpOrAsn = ExtensionPreferences & {}
}

declare namespace Arguments {
  /** Arguments passed to the `lookup-ip-or-asn` command */
  export type LookupIpOrAsn = {
  /** IP_OR_ASN */
  "ip": string
}
}

