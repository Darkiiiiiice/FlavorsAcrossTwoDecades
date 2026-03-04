//! 资金系统

use serde::{Deserialize, Serialize};

/// 资金管理
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finance {
    /// 现金
    pub cash: u64,
    /// 今日收入
    pub daily_revenue: u64,
    /// 今日支出
    pub daily_expenses: u64,
    /// 总收入
    pub total_revenue: u64,
    /// 总支出
    pub total_expenses: u64,
}

impl Finance {
    /// 创建新的资金系统
    pub fn new() -> Self {
        Self {
            cash: 10000, // 初始资金
            daily_revenue: 0,
            daily_expenses: 0,
            total_revenue: 0,
            total_expenses: 0,
        }
    }

    /// 添加收入
    pub fn add_revenue(&mut self, amount: u64) {
        self.daily_revenue += amount;
        self.total_revenue += amount;
        self.cash += amount;
    }

    /// 添加支出
    pub fn add_expense(&mut self, amount: u64) {
        self.daily_expenses += amount;
        self.total_expenses += amount;
        self.cash = self.cash.saturating_sub(amount);
    }

    /// 计算利润率
    pub fn profit_rate(&self) -> f32 {
        if self.total_revenue == 0 {
            return 0.0;
        }
        (self.total_revenue - self.total_expenses) as f32 / self.total_revenue as f32
    }

    /// 计算今日净利润
    pub fn daily_profit(&self) -> i64 {
        self.daily_revenue as i64 - self.daily_expenses as i64
    }

    /// 重置每日账目
    pub fn reset_daily(&mut self) {
        self.daily_revenue = 0;
        self.daily_expenses = 0;
    }

    /// 检查是否可以支付
    pub fn can_afford(&self, amount: u64) -> bool {
        self.cash >= amount
    }

    /// 支付
    pub fn pay(&mut self, amount: u64) -> Result<(), String> {
        if !self.can_afford(amount) {
            return Err("资金不足".to_string());
        }
        self.add_expense(amount);
        Ok(())
    }
}

impl Default for Finance {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finance_creation() {
        let finance = Finance::new();
        assert_eq!(finance.cash, 10000);
        assert_eq!(finance.daily_revenue, 0);
    }

    #[test]
    fn test_revenue() {
        let mut finance = Finance::new();
        finance.add_revenue(500);

        assert_eq!(finance.daily_revenue, 500);
        assert_eq!(finance.total_revenue, 500);
        assert_eq!(finance.cash, 10500);
    }

    #[test]
    fn test_expense() {
        let mut finance = Finance::new();
        finance.add_expense(200);

        assert_eq!(finance.daily_expenses, 200);
        assert_eq!(finance.total_expenses, 200);
        assert_eq!(finance.cash, 9800);
    }

    #[test]
    fn test_profit_rate() {
        let mut finance = Finance::new();

        // 无收入
        assert_eq!(finance.profit_rate(), 0.0);

        // 有收入和支出
        finance.add_revenue(1000);
        finance.add_expense(600);
        assert_eq!(finance.profit_rate(), 0.4); // (1000-600)/1000
    }

    #[test]
    fn test_payment() {
        let mut finance = Finance::new();

        // 可以支付
        assert!(finance.pay(5000).is_ok());
        assert_eq!(finance.cash, 5000);

        // 不能支付
        assert!(finance.pay(10000).is_err());
    }
}
