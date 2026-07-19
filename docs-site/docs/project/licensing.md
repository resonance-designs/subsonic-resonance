---
title: Licensing guide
sidebar_position: 4
---

<!-- Generated from LICENSING.md by scripts/sync-docs.js. Do not edit directly. -->

# Licensing guide

Last reviewed: July 18, 2026

## Purpose

This document records the intended licensing model for Resonance, the licensing implications of supporting Subsonic and OpenSubsonic, and the work required before distributing the application.

It is a project reference, not a license agreement or legal advice. Final end-user, confidentiality, and source-access agreements should be prepared or reviewed by a qualified attorney for the jurisdictions in which Resonance will be distributed.

## Intended distribution model

Resonance may be distributed as **proprietary freeware**:

- Anyone may download and use official compiled releases without paying for the application.
- The Resonance source code remains private and is not distributed with ordinary releases.
- Receiving a free binary does not grant a right to receive, modify, or redistribute its source code.
- A developer who needs access to the proprietary source must enter into a separate written source-access agreement and confidentiality agreement and may be charged for that access.
- Rights to modify, build, distribute, sublicense, or commercialize Resonance are granted only when the applicable agreement expressly says so.

In this context, Resonance should be described as **free to use** or **available at no charge**. It should not be described as “free software” or “open source,” because those terms normally indicate that recipients receive source-code freedoms that this model does not grant.

Charging nothing for the compiled application does not prevent it from being proprietary software. Price and source-code rights are separate decisions.

## Subsonic and OpenSubsonic compatibility

### OpenSubsonic specification

The OpenSubsonic API specification and documentation repository is licensed under the Apache License 2.0. Apache-2.0 permits commercial and proprietary use, modification, and distribution, subject to its notice, attribution, modification-notice, and other conditions when licensed material is redistributed.

Resonance independently implements the documented HTTP protocol. Merely sending compatible requests to an OpenSubsonic server does not require Resonance to adopt the specification's license or publish its own source code.

If text, schemas, examples, generated code, or other material is copied from the OpenSubsonic repository into Resonance, the copied material must be reviewed separately and all applicable Apache-2.0 notice and attribution requirements must be preserved.

References:

- [OpenSubsonic documentation](https://opensubsonic.netlify.app/docs/)
- [OpenSubsonic specification repository](https://github.com/opensubsonic/open-subsonic-api)
- [OpenSubsonic Apache-2.0 license](https://github.com/opensubsonic/open-subsonic-api/blob/main/LICENSE)

### Original Subsonic software

Historical Subsonic implementations have had their own software licensing terms. Those terms matter if Resonance copies, modifies, links to, embeds, or redistributes code from one of those implementations.

Resonance currently communicates with compatible servers over the documented network API and does not embed the original Subsonic server. An independently written client that implements the protocol is separate from the server implementation and does not ordinarily inherit the server's source-code license merely because the two applications communicate.

The official Subsonic documentation states that REST API access to an official Subsonic server requires a valid server license after its trial period. That concerns use of the official server installation; it does not state that an independently implemented client must be open source.

Reference:

- [Official Subsonic API documentation](https://www.subsonic.org/pages/api.jsp)

### Compatibility naming and trademarks

Protocol compatibility does not automatically grant trademark rights. Product copy should use factual compatibility language such as “compatible with Subsonic and OpenSubsonic servers.” Resonance should not imply that it is produced, sponsored, certified, or endorsed by Subsonic or OpenSubsonic unless permission has been obtained.

## End-user binary license

Official compiled releases should be accompanied by a proprietary end-user license agreement, or EULA. The EULA should state at minimum:

- Who owns Resonance and its copyrighted components.
- That the user receives a limited license to install and use compiled releases at no charge.
- Whether personal, educational, nonprofit, and business use are all permitted.
- Whether copying installers for internal use or redistribution from third-party sites is permitted.
- Restrictions on modification, sublicensing, resale, and unauthorized source disclosure.
- Any reverse-engineering restrictions, subject to exceptions required by applicable law.
- Warranty disclaimers and limitations of liability.
- Termination terms and the governing law or dispute process.
- Which privacy policy and service terms apply to network features, if any.
- That third-party components remain governed by their respective licenses.

The final EULA should be drafted or reviewed by an attorney. A project note should not be treated as a substitute for an executed agreement.

## Paid source access

Source access should be controlled through a separately executed agreement rather than through the end-user EULA. Depending on the relationship, this may consist of an NDA plus a source-code evaluation, development, or commercial license.

The agreement should define:

- The person or company receiving access and the authorized developers.
- The repositories, branches, documentation, build systems, and other confidential material covered by the agreement.
- The price, payment schedule, access period, and renewal terms.
- Permitted purposes, such as evaluation, integration, contracted development, internal customization, or creation of an approved product.
- Whether the developer may compile private builds or distribute modified binaries.
- Whether plugins and integrations may be distributed independently.
- A prohibition on publishing, sublicensing, transferring, or otherwise disclosing proprietary source unless expressly authorized.
- Storage, access-control, incident-reporting, and source-deletion requirements.
- Whether subcontractors may receive access and, if so, under which protections.
- Ownership of modifications, bug fixes, documentation, and other work product.
- Treatment of the developer's pre-existing intellectual property.
- Whether changes are assigned to the Resonance owner or licensed back with sufficiently broad, perpetual rights.
- Patent, trademark, feedback, and derivative-work rights.
- Whether and how compliance may be audited.
- Termination, return or destruction of source, breach remedies, and continuing confidentiality obligations.
- Restrictions on competing uses only where they are lawful and appropriately tailored.

An NDA alone is usually not enough. An NDA can protect confidentiality, but it does not necessarily grant the developer permission to modify, compile, or distribute the software. The source-code license or development agreement must grant those rights explicitly.

## Contributions and ownership

The ability to offer proprietary source access depends on having the necessary rights to the entire codebase.

Before accepting outside contributions, the project should adopt one of these approaches:

1. Require contributors to assign copyright in their contributions to the Resonance owner.
2. Require a contributor license agreement granting the Resonance owner perpetual, worldwide, irrevocable rights to use, modify, sublicense, relicense, and commercially distribute the contribution, including in closed-source products.
3. Accept work only under written contractor or employment terms that clearly assign the resulting intellectual property.

A simple pull request does not necessarily provide all rights required to relicense that contribution as part of a proprietary product. Contribution terms should be presented and accepted before code is merged, and records of acceptance should be retained.

Contributors must also confirm that their changes are original or appropriately licensed and do not contain copied GPL, AGPL, confidential, employer-owned, or otherwise incompatible code.

## Optional public SDK and plugin model

Resonance may publish a separate SDK, provider interface, or plugin-development kit under a permissive license such as MIT or Apache-2.0. This would allow developers to create integrations without gaining access to the proprietary application core.

To preserve that boundary:

- Keep the public SDK in clearly separated packages or repositories.
- Give it its own explicit license and copyright notices.
- Document the stable API exposed to plugins.
- Avoid requiring plugins to copy proprietary implementation code.
- Define plugin signing, compatibility, distribution, and trademark rules separately.

Developers who need to alter the core would still require the paid source-access arrangement.

## Browser and WebAssembly considerations

The browser client necessarily sends WebAssembly, JavaScript glue, HTML, CSS, images, and other assets to the user's device. Users can inspect, copy, and reverse-engineer those artifacts even when the preferred Rust source is private.

A proprietary license may restrict some uses where legally enforceable, but licensing cannot make client-delivered artifacts technically secret. Sensitive credentials, private keys, enforcement logic, and commercially sensitive algorithms should not be embedded in browser assets. They should remain in the local Rust backend, native process, or another appropriately protected component.

Debug symbols, source maps, development endpoints, build paths, and embedded secrets must be excluded from production browser and desktop releases.

## Current repository alignment

The root `Cargo.toml` currently declares:

```toml
[workspace.package]
license = "MIT OR Apache-2.0"
```

That declaration conflicts with the intended proprietary-freeware model. MIT and Apache-2.0 allow recipients to copy, modify, redistribute, sublicense, and use covered code in proprietary products subject to their respective conditions.

Before a proprietary release or intentional source distribution, the project should:

1. Confirm whether any copies or commits have already been intentionally offered under MIT or Apache-2.0.
2. Confirm that the current owner has the right to relicense every contribution.
3. Replace the workspace declaration with an appropriate proprietary identifier such as `LicenseRef-Proprietary`, where supported by the applicable Cargo publishing workflow.
4. Add a top-level proprietary `LICENSE` or copyright-and-license notice.
5. Add the final end-user EULA.
6. Add a third-party notices document and corresponding release-generation process.
7. Keep the source repository private and control developer access.
8. Review package registries, source archives, CI artifacts, container images, and release bundles so source is not published unintentionally.

Changing the license affects future distributions. It does not revoke rights already granted for copies previously distributed under MIT or Apache-2.0. Those recipients may continue exercising the license applicable to the copies they received.

No licensing metadata should be changed until ownership and prior-distribution history have been confirmed.

## Third-party dependencies

Resonance depends on Rust crates, native libraries, web assets, fonts, icons, codecs, and platform components that retain their own licenses. A proprietary license for Resonance does not replace those licenses.

The dependency set reviewed on July 18, 2026 was predominantly under permissive licenses including MIT, Apache-2.0, BSD, ISC, Zlib, Unicode, and similar terms. Some transitive packages reported MPL-2.0 or multi-license expressions. No dependency in that metadata review obviously required Resonance as a whole to be distributed under the GPL, but package metadata alone is not a complete legal audit.

Before every release:

- Generate a dependency inventory from the locked dependency graph for every shipped target.
- Resolve every multi-license expression using an option compatible with proprietary distribution.
- Include required copyright, license, attribution, and NOTICE texts.
- Review MPL, LGPL, codec, font, artwork, and native system-library obligations individually.
- Check bundled binaries and installer payloads, not only direct Rust dependencies.
- Confirm that optional and target-specific dependencies used by Windows, macOS, Linux, WebAssembly, and mobile builds are covered.
- Record dependency versions and licenses in a reproducible software bill of materials.
- Block dependencies with incompatible or unknown licensing until reviewed.

Automated tools such as `cargo-deny`, `cargo-about`, or an equivalent audited process may assist with inventory and notice generation, but automated metadata should be verified against each dependency's actual license files.

## Release checklist

Before distributing Resonance under this model:

- [ ] Confirm ownership of all Resonance source and assets.
- [ ] Determine whether any version was already distributed under MIT or Apache-2.0.
- [ ] Replace the current workspace licensing metadata after that review.
- [ ] Add the proprietary copyright and license notice.
- [ ] Obtain legal review of the EULA.
- [ ] Obtain legal review of the NDA and paid source-access agreement.
- [ ] Adopt contribution and contractor ownership terms.
- [ ] Establish a private repository access and offboarding process.
- [ ] Complete a locked dependency and bundled-asset license audit.
- [ ] Generate and ship third-party notices and an SBOM.
- [ ] Review compatibility statements for trademark accuracy.
- [ ] Remove source maps, debug symbols, secrets, and development artifacts as appropriate.
- [ ] Verify that installers and update packages contain only intended distributable files.
- [ ] Establish a process for responding to third-party license requests and security reports.
- [ ] Repeat the dependency and notice review for every release and target platform.

## Summary

Resonance can be made available to users at no charge while remaining proprietary and closed-source. Supporting Subsonic and OpenSubsonic through an independently implemented network client does not, by itself, require publishing the Resonance source code.

Ordinary users would receive a limited license to run compiled releases. Developers needing proprietary source would require a paid, written source-access license plus appropriate confidentiality protections. A public, permissively licensed plugin SDK could be offered separately without opening the application core.

The current `MIT OR Apache-2.0` project declaration must be resolved before adopting this model, and third-party dependency obligations must continue to be honored regardless of the license selected for Resonance itself.
