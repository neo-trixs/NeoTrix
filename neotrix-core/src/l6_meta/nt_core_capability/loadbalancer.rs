//! 能力负载均衡
//!
//! 智能负载均衡和故障转移

use std::time::{Duration, Instant};

/// 负载均衡策略
#[derive(Debug, Clone)]
pub enum LoadBalanceStrategy {
    /// 轮询
    RoundRobin,
    /// 加权轮询
    WeightedRoundRobin,
    /// 最少连接
    LeastConnections,
    /// 加权最少连接
    WeightedLeastConnections,
    /// 响应时间
    ResponseTime,
    /// 资源使用率
    ResourceUtilization,
}

/// 能力实例
#[derive(Debug, Clone)]
pub struct CapabilityInstance {
    /// 实例ID
    pub id: String,
    /// 能力ID
    pub capability_id: String,
    /// 权重
    pub weight: u32,
    /// 当前连接数
    pub current_connections: u32,
    /// 最大连接数
    pub max_connections: u32,
    /// 平均响应时间
    pub avg_response_time_ms: u64,
    /// 成功率
    pub success_rate: f64,
    /// 总调用次数
    pub total_calls: u64,
    /// 最后调用时间
    pub last_called: Option<Instant>,
    /// 状态
    pub status: InstanceStatus,
}

/// 实例状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstanceStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Maintenance,
}

/// 负载均衡器
pub struct LoadBalancer {
    /// 策略
    strategy: LoadBalanceStrategy,
    /// 能力实例
    instances: Vec<CapabilityInstance>,
    /// 轮询索引
    round_robin_index: usize,
    /// 统计信息
    stats: LoadBalancerStats,
}

/// 负载均衡统计
#[derive(Debug, Clone, Default)]
pub struct LoadBalancerStats {
    /// 总请求数
    pub total_requests: u64,
    /// 成功请求数
    pub successful_requests: u64,
    /// 失败请求数
    pub failed_requests: u64,
    /// 平均响应时间
    pub avg_response_time_ms: f64,
    /// 活跃实例数
    pub active_instances: usize,
    /// 总实例数
    pub total_instances: usize,
}

impl LoadBalancer {
    /// 创建新的负载均衡器
    pub fn new(strategy: LoadBalanceStrategy) -> Self {
        Self {
            strategy,
            instances: Vec::new(),
            round_robin_index: 0,
            stats: LoadBalancerStats::default(),
        }
    }

    /// 添加实例
    pub fn add_instance(&mut self, instance: CapabilityInstance) {
        self.instances.push(instance);
        self.stats.total_instances = self.instances.len();
        self.update_active_instances();
    }

    /// 移除实例
    pub fn remove_instance(&mut self, instance_id: &str) {
        self.instances.retain(|i| i.id != instance_id);
        self.stats.total_instances = self.instances.len();
        self.update_active_instances();
    }

    /// 选择实例
    pub fn select_instance(&mut self, capability_id: &str) -> Option<&CapabilityInstance> {
        let available: Vec<_> = self
            .instances
            .iter()
            .filter(|i| i.capability_id == capability_id && i.status == InstanceStatus::Healthy)
            .collect();

        if available.is_empty() {
            return None;
        }

        match self.strategy {
            LoadBalanceStrategy::RoundRobin => {
                let index = self.round_robin_index % available.len();
                self.round_robin_index += 1;
                Some(available[index])
            }
            LoadBalanceStrategy::WeightedRoundRobin => {
                // 加权轮询
                let total_weight: u32 = available.iter().map(|i| i.weight).sum();
                if total_weight == 0 {
                    return None;
                }
                let mut random = (chrono::Utc::now().timestamp_millis() as u32) % total_weight;
                for instance in &available {
                    if random < instance.weight {
                        return Some(instance);
                    }
                    random -= instance.weight;
                }
                available.last().copied()
            }
            LoadBalanceStrategy::LeastConnections => available
                .iter()
                .min_by_key(|i| i.current_connections)
                .copied(),
            LoadBalanceStrategy::WeightedLeastConnections => {
                // 加权最少连接
                available
                    .iter()
                    .min_by(|a, b| {
                        let score_a = a.current_connections as f64 / a.weight as f64;
                        let score_b = b.current_connections as f64 / b.weight as f64;
                        score_a
                            .partial_cmp(&score_b)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .copied()
            }
            LoadBalanceStrategy::ResponseTime => available
                .iter()
                .min_by_key(|i| i.avg_response_time_ms)
                .copied(),
            LoadBalanceStrategy::ResourceUtilization => available
                .iter()
                .min_by(|a, b| {
                    let util_a = a.current_connections as f64 / a.max_connections as f64;
                    let util_b = b.current_connections as f64 / b.max_connections as f64;
                    util_a
                        .partial_cmp(&util_b)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .copied(),
        }
    }

    /// 更新实例统计
    pub fn update_instance_stats(
        &mut self,
        instance_id: &str,
        response_time_ms: u64,
        success: bool,
    ) {
        if let Some(instance) = self.instances.iter_mut().find(|i| i.id == instance_id) {
            instance.total_calls += 1;
            instance.last_called = Some(Instant::now());

            // 更新平均响应时间
            let total_time =
                instance.avg_response_time_ms * (instance.total_calls - 1) + response_time_ms;
            instance.avg_response_time_ms = total_time / instance.total_calls;

            // 更新成功率
            let total_success = (instance.success_rate * (instance.total_calls - 1) as f64) as u64;
            let new_success = if success {
                total_success + 1
            } else {
                total_success
            };
            instance.success_rate = new_success as f64 / instance.total_calls as f64;

            // 更新状态
            if instance.success_rate < 0.5 {
                instance.status = InstanceStatus::Unhealthy;
            } else if instance.success_rate < 0.8 {
                instance.status = InstanceStatus::Degraded;
            } else {
                instance.status = InstanceStatus::Healthy;
            }
        }

        // 更新统计
        self.stats.total_requests += 1;
        if success {
            self.stats.successful_requests += 1;
        } else {
            self.stats.failed_requests += 1;
        }
        self.update_avg_response_time(response_time_ms);
    }

    /// 更新平均响应时间
    fn update_avg_response_time(&mut self, new_time: u64) {
        let total_requests = self.stats.total_requests;
        if total_requests == 1 {
            self.stats.avg_response_time_ms = new_time as f64;
        } else {
            self.stats.avg_response_time_ms =
                (self.stats.avg_response_time_ms * (total_requests - 1) as f64 + new_time as f64)
                    / total_requests as f64;
        }
    }

    /// 更新活跃实例数
    fn update_active_instances(&mut self) {
        self.stats.active_instances = self
            .instances
            .iter()
            .filter(|i| i.status == InstanceStatus::Healthy)
            .count();
    }

    /// 获取统计信息
    pub fn stats(&self) -> &LoadBalancerStats {
        &self.stats
    }

    /// 获取所有实例
    pub fn get_instances(&self) -> &[CapabilityInstance] {
        &self.instances
    }

    /// 获取指定能力的实例
    pub fn get_capability_instances(&self, capability_id: &str) -> Vec<&CapabilityInstance> {
        self.instances
            .iter()
            .filter(|i| i.capability_id == capability_id)
            .collect()
    }

    /// 故障转移
    pub fn failover(
        &mut self,
        failed_instance_id: &str,
        capability_id: &str,
    ) -> Option<&CapabilityInstance> {
        // 标记失败实例
        if let Some(instance) = self
            .instances
            .iter_mut()
            .find(|i| i.id == failed_instance_id)
        {
            instance.status = InstanceStatus::Unhealthy;
        }

        // 选择备用实例
        self.select_instance(capability_id)
    }

    /// 健康检查
    pub fn health_check(&mut self) -> Vec<String> {
        let mut unhealthy = Vec::new();

        for instance in &mut self.instances {
            if let Some(last_called) = instance.last_called {
                let elapsed = last_called.elapsed();
                if elapsed > Duration::from_secs(300) && instance.status == InstanceStatus::Healthy
                {
                    instance.status = InstanceStatus::Degraded;
                    unhealthy.push(instance.id.clone());
                }
            }
        }

        self.update_active_instances();
        unhealthy
    }

    /// 清理不健康实例
    pub fn cleanup_unhealthy(&mut self, max_age: Duration) {
        let cutoff = Instant::now() - max_age;
        self.instances.retain(|i| {
            i.status != InstanceStatus::Unhealthy
                || i.last_called.map(|t| t > cutoff).unwrap_or(false)
        });
        self.stats.total_instances = self.instances.len();
        self.update_active_instances();
    }
}

impl Default for LoadBalancer {
    fn default() -> Self {
        Self::new(LoadBalanceStrategy::RoundRobin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_balancer_creation() {
        let lb = LoadBalancer::new(LoadBalanceStrategy::RoundRobin);
        assert_eq!(lb.stats().total_instances, 0);
    }

    #[test]
    fn add_instance() {
        let mut lb = LoadBalancer::new(LoadBalanceStrategy::RoundRobin);
        let instance = CapabilityInstance {
            id: "inst_1".into(),
            capability_id: "cap_1".into(),
            weight: 1,
            current_connections: 0,
            max_connections: 100,
            avg_response_time_ms: 0,
            success_rate: 1.0,
            total_calls: 0,
            last_called: None,
            status: InstanceStatus::Healthy,
        };

        lb.add_instance(instance);
        assert_eq!(lb.stats().total_instances, 1);
    }

    #[test]
    fn select_instance() {
        let mut lb = LoadBalancer::new(LoadBalanceStrategy::RoundRobin);
        let instance = CapabilityInstance {
            id: "inst_1".into(),
            capability_id: "cap_1".into(),
            weight: 1,
            current_connections: 0,
            max_connections: 100,
            avg_response_time_ms: 0,
            success_rate: 1.0,
            total_calls: 0,
            last_called: None,
            status: InstanceStatus::Healthy,
        };

        lb.add_instance(instance);
        let selected = lb.select_instance("cap_1");
        assert!(selected.is_some());
    }

    #[test]
    fn failover() {
        let mut lb = LoadBalancer::new(LoadBalanceStrategy::RoundRobin);
        let instance = CapabilityInstance {
            id: "inst_1".into(),
            capability_id: "cap_1".into(),
            weight: 1,
            current_connections: 0,
            max_connections: 100,
            avg_response_time_ms: 0,
            success_rate: 1.0,
            total_calls: 0,
            last_called: None,
            status: InstanceStatus::Healthy,
        };

        lb.add_instance(instance);
        let backup = lb.failover("inst_1", "cap_1");
        assert!(backup.is_none()); // 没有备用实例
    }
}
