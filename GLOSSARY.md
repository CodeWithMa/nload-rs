# Network Traffic Monitor

The language of viewing network traffic one interface at a time.

## Language

**Interface**:
A network interface whose incoming and outgoing traffic is monitored separately.
_Avoid_: Device (except when quoting upstream labels or documentation).

**Single-interface view**:
The screen showing traffic for one selected interface, with navigation to other interfaces.
_Avoid_: Multi-interface view, dashboard.

**Current rate**:
The incoming or outgoing traffic rate over the most recent one-second window, using the shorter available history during warm-up.

**Average rate**:
The incoming or outgoing traffic rate over the most recent five-minute window, using the shorter available history during warm-up.

**Cumulative total**:
The latest interface-accounted incoming or outgoing byte count, whose starting point is determined by the interface's accounting lifetime.
_Avoid_: Since-boot total, session total.

**Graph ceiling**:
The traffic rate represented by a full-height graph; higher rates saturate the graph without capping the numeric rate.

**Graph sample**:
A current-rate measurement represented by one column in an incoming or outgoing traffic graph.

**Graph history**:
The retained sequence of graph samples for an interface.

**Traffic observation**:
A reading of an interface's incoming and outgoing cumulative byte counts at an observation time.

**Baseline observation**:
The first traffic observation of a statistics run, providing cumulative totals but not yet a measured rate.

**Unavailable interface**:
A retained interface entry whose traffic counters cannot currently be observed, because the interface is absent or counter acquisition has failed.

**Statistics run**:
A continuous period of monitoring an interface between statistics resets, beginning with a baseline observation.

**Minimum rate**:
The lowest measured current rate in a statistics run, independently for incoming and outgoing traffic.

**Maximum rate**:
The highest measured current rate in a statistics run, independently for incoming and outgoing traffic.
