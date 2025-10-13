# Jožin Worker — Thumbnails and image adjustments (jozimg)

## Purpose
The **jozimg worker** is responsible for generating and maintaining **thumbnails and preview images** within the Jožin ecosystem. These previews will be used by the GUI and other components for fast browsing and sharing without exposing original files.

## Planned Functionality
- Create and cache thumbnails in various predefined sizes.
- Support both raster and RAW formats via embedded JPEGs or conversions.
- Maintain aspect ratio and orientation consistency (EXIF-aware).
- Optionally generate blurred or low-quality preview variants for privacy-preserving sharing.
- Store thumbnails in a structured local cache or database-linked storage path.

## Status
Detailed functional and technical specification **TBD**.

This document serves as a **template placeholder** for future expansion once the thumbnail generation and caching pipeline is defined.
