@0x8d8d820af6c5d028;

struct StatsdBatch {
  metricLabels @0 :List(Text);
  metricValues @1 :List(Float64);
  metricKinds @2 :List(MetricKind);

  enum MetricKind {
     gauge @0;
     counter @1;
     timer @2;
  }
}

