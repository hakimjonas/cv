# Metrics for Measuring Impact

This document outlines the metrics implementation for measuring the impact of improvements made to the CV and Blog application. It defines key metrics, establishes baseline measurements, and describes how these metrics will be collected, monitored, and analyzed.

## Table of Contents

1. [Key Metrics](#key-metrics)
2. [Baseline Measurements](#baseline-measurements)
3. [Integration with Monitoring System](#integration-with-monitoring-system)
4. [Dashboards](#dashboards)
5. [Alerting](#alerting)
6. [Implementation Plan](#implementation-plan)

## Key Metrics

We have identified the following key metrics to measure the impact of our improvements:

### Performance Metrics

| Metric | Description | Target | Collection Method |
|--------|-------------|--------|-------------------|
| API Response Time | Average time to respond to API requests | < 100ms | Prometheus metrics via middleware |
| Page Load Time | Time to load the main pages (CV, Blog) | < 1s | Browser performance API + Prometheus |
| Database Query Time | Average time for database queries | < 50ms | Prometheus metrics via database middleware |
| Memory Usage | Application memory consumption | < 200MB | Node exporter + Prometheus |
| CPU Usage | Application CPU utilization | < 30% | Node exporter + Prometheus |

### Reliability Metrics

| Metric | Description | Target | Collection Method |
|--------|-------------|--------|-------------------|
| Uptime | Percentage of time the application is available | > 99.9% | Prometheus uptime monitoring |
| Error Rate | Percentage of requests that result in errors | < 0.1% | Prometheus metrics via middleware |
| Database Connection Failures | Number of failed database connections | < 5 per day | Database pool metrics |
| Successful Deployments | Percentage of deployments without rollbacks | > 95% | CI/CD metrics |
| Mean Time to Recovery (MTTR) | Average time to recover from failures | < 30 minutes | Incident management system |

### Usage Metrics

| Metric | Description | Target | Collection Method |
|--------|-------------|--------|-------------------|
| Page Views | Number of page views per day | Increasing trend | Application logs + Prometheus |
| API Requests | Number of API requests per day | Increasing trend | Application logs + Prometheus |
| Unique Visitors | Number of unique visitors per day | Increasing trend | Application logs + Prometheus |
| Session Duration | Average time users spend on the site | > 2 minutes | Browser events + Prometheus |
| Bounce Rate | Percentage of visitors who leave after viewing only one page | < 40% | Browser events + Prometheus |

### Security Metrics

| Metric | Description | Target | Collection Method |
|--------|-------------|--------|-------------------|
| Failed Authentication Attempts | Number of failed login attempts | < 10 per day | Application logs + Prometheus |
| Security Vulnerabilities | Number of identified security vulnerabilities | 0 | Security scanning tools |
| Time to Patch | Time to patch identified vulnerabilities | < 48 hours | Incident management system |
| CSRF Attempts | Number of detected CSRF attempts | 0 | Application logs + Prometheus |
| Rate Limit Breaches | Number of rate limit breaches | < 5 per day | Application logs + Prometheus |

## Baseline Measurements

Before implementing improvements, we will establish baseline measurements for all metrics to enable accurate comparison and impact assessment. The baseline measurement process will:

1. Collect data for each metric over a 2-week period
2. Calculate average values and standard deviations
3. Document the baseline values in a baseline report
4. Set up comparison dashboards to track improvements against the baseline

## Integration with Monitoring System

We will integrate our metrics collection with the existing Prometheus and Grafana monitoring system:

### Prometheus Configuration

```yaml
# Add to prometheus.yml
scrape_configs:
  - job_name: 'blog-api'
    metrics_path: '/metrics'
    static_configs:
      - targets: ['blog-api:3000']
    scrape_interval: 15s
    
  - job_name: 'blog-frontend'
    metrics_path: '/metrics'
    static_configs:
      - targets: ['blog-frontend:80']
    scrape_interval: 15s
```

### Application Instrumentation

We will add instrumentation to the application code to collect metrics:

1. **API Response Time**: Add middleware to measure request duration
2. **Database Query Time**: Add instrumentation to database queries
3. **Error Rate**: Add middleware to count and categorize errors
4. **Usage Metrics**: Add logging for page views, API requests, and user sessions

Example middleware for measuring API response time:

```rust
async fn metrics_middleware<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let path = req.uri().path().to_owned();
    let method = req.method().clone();
    
    let start = Instant::now();
    let response = next.run(req).await;
    let duration = start.elapsed();
    
    let status = response.status().as_u16();
    
    // Record metrics in Prometheus
    HTTP_REQUESTS_TOTAL
        .with_label_values(&[&path, method.as_str(), &status.to_string()])
        .inc();
        
    HTTP_REQUESTS_DURATION
        .with_label_values(&[&path, method.as_str(), &status.to_string()])
        .observe(duration.as_secs_f64());
        
    Ok(response)
}
```

## Dashboards

We will create the following Grafana dashboards to visualize metrics:

### 1. Performance Dashboard

- API response time by endpoint (histogram)
- Page load time by page (histogram)
- Database query time by query type (histogram)
- Memory and CPU usage over time (line chart)
- Request rate by endpoint (line chart)

### 2. Reliability Dashboard

- Uptime percentage (gauge)
- Error rate by endpoint (line chart)
- Database connection failures (line chart)
- Deployment success rate (gauge)
- MTTR over time (line chart)

### 3. Usage Dashboard

- Page views by page (line chart)
- API requests by endpoint (line chart)
- Unique visitors over time (line chart)
- Session duration distribution (histogram)
- Bounce rate over time (line chart)

### 4. Security Dashboard

- Failed authentication attempts (line chart)
- Security vulnerabilities (table)
- Time to patch vulnerabilities (gauge)
- CSRF attempts (line chart)
- Rate limit breaches (line chart)

## Alerting

We will configure alerts for critical metrics to ensure timely response to issues:

### Alert Rules

```yaml
# Add to prometheus/rules/alerts.yml
groups:
  - name: blog-api-alerts
    rules:
      - alert: HighErrorRate
        expr: sum(rate(http_requests_total{status=~"5.."}[5m])) / sum(rate(http_requests_total[5m])) > 0.01
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "Error rate is above 1% for the last 5 minutes"
          
      - alert: SlowAPIResponse
        expr: histogram_quantile(0.95, sum(rate(http_requests_duration_bucket[5m])) by (le, path)) > 0.5
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Slow API response detected"
          description: "95th percentile response time is above 500ms for the last 5 minutes"
          
      - alert: HighMemoryUsage
        expr: process_resident_memory_bytes / 1024 / 1024 > 200
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage detected"
          description: "Memory usage is above 200MB for the last 5 minutes"
```

### Notification Channels

We will configure the following notification channels in Grafana:

1. **Slack**: For immediate team notifications
2. **Email**: For daily/weekly summary reports
3. **PagerDuty**: For critical alerts requiring immediate attention

## Implementation Plan

### Phase 1: Setup (Week 1)

1. Configure Prometheus to collect metrics from the application
2. Add instrumentation to the application code
3. Create initial Grafana dashboards
4. Establish baseline measurements

### Phase 2: Refinement (Week 2)

1. Refine metrics collection based on initial data
2. Enhance dashboards with additional visualizations
3. Configure alerting rules
4. Set up notification channels

### Phase 3: Automation (Week 3)

1. Automate regular reporting of metrics
2. Implement anomaly detection for key metrics
3. Create documentation for the metrics system
4. Train team members on using the metrics dashboards

### Phase 4: Continuous Improvement (Ongoing)

1. Regularly review metrics to identify areas for improvement
2. Adjust alert thresholds based on operational experience
3. Add new metrics as needed
4. Generate periodic impact reports comparing current metrics to baseline

## Conclusion

By implementing this metrics system, we will be able to:

1. Quantify the impact of our improvements
2. Identify areas that need further optimization
3. Detect and respond to issues quickly
4. Make data-driven decisions about future improvements

The metrics will provide objective evidence of the value delivered by our work and guide our ongoing efforts to enhance the application.