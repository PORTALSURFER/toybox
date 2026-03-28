//! Strict declarative Patchbay GUI re-exports for plugin UIs.
//!
//! The module exposes the strict declarative GUI surface so plugin crates can
//! build UI specs and reduce typed actions without depending on `patchbay-gui`
//! directly. Immediate-mode authoring APIs are intentionally not re-exported.

pub use patchbay_gui::{
    Canvas, Color, InputState, MainPalette, Point, Rect, RenderedFrame, Size, render_spec_to_frame,
};

/// Reusable waveform view rendering helpers for declarative surfaces.
pub mod waveform;

/// Strict declarative GUI types and rendering helpers.
pub mod declarative {
    pub use patchbay_gui::{
        AccentKey, Align, ButtonSpec, ColorStateVariants, ColorTokens, ContainerLayout,
        ContainerLength, ControlTokens, CurveEditorSpec, CurveEditorStyle, CurveGridConfig,
        CurveHighlightMode, CurveInteractionOptions, CurveModel, CurvePoint, CurveSegment,
        CurveSnapConfig, DeclarativeError, DefaultWidgetColorResolver, DropdownSpec, EdgeInsets,
        EndpointMode, EqAttractor, EqAttractorColorPolicy, EqAttractorSurfaceAction,
        EqAttractorSurfaceModel, EqAttractorSurfaceSpec, EqAttractorSurfaceStyle, FlexSpec,
        GridKind, GridSpec, GridTemplate, IndicatorSpec, Justify, KnobSpec, LayoutBox,
        LayoutContainerKind, LayoutDiagnostic, LayoutDiagnosticCode, LayoutDiagnosticLevel,
        LayoutDiagnosticsMode, LayoutEngineState, LayoutNodeDiagnostic, LayoutNodeDiagnosticReason,
        LayoutNodeKind, LayoutOverflowSummary, Length, MainPalette, MeasureCacheKey,
        MeasureCacheStats, Node, NodeId, OverflowPolicy, PanelSpec, RegionInteractionKind,
        RenderResult, RootFrameSpec, RootScaleMode, ScrollViewSpec, SemanticColorToken, SliderSpec,
        Slot, SlotAlign, SlotCrossSize, SlotMainSize, SlotParams, SlotSpec, SpacingTokens,
        StackSpec, SurfaceCommand, SwitchCase, SwitchLayoutSpec, SwitchWidthRange, TabBarSpec,
        TextBoxSpec, ThemeTokens, ToggleSpec, TrackSize, TypographyTokens, UiAction,
        UiInvalidationScope, UiSpec, WeightedSlot, WidgetColorContext, WidgetColorResolver,
        WidgetColorRole, WidgetColorToken, WrapSpec, button, column, column_slots, curve_editor,
        dropdown, eq_attractor_surface, fill_slot, fraction_slot, grid, indicator, knob,
        measure_checked, panel, region, render_checked, render_checked_with_engine,
        root_frame_sized, row, row_slots, scroll_view, slider, slot, spacer, stack, surface,
        switch_layout, tabbar, textbox, toggle, weighted_slot, weighted_slot_lengths,
        when_width_between, when_width_ge, when_width_lt, wrap,
    };
}

/// Screenshot test helpers for declarative UIs.
///
/// This module is feature-gated so plugins can opt into screenshot testing
/// without pulling screenshot-only dependencies into release builds.
#[cfg(feature = "screenshot-test")]
pub mod screenshot_harness;
