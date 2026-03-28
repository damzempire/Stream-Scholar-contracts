/**
 * GasEstimationService for Stream Scholar
 * 
 * This service provides utility functions to estimate the total XLM gas fees required 
 * for a student to watch a course of a specific duration. It takes into account
 * the initial 'buy_access' transaction and the subsequent periodic 'heartbeat' transactions.
 */
export class GasEstimationService {
  /**
   * Average gas fee for a 'buy_access' transaction in XLM.
   * Based on current Soroban network parameters for a transaction involving 
   * authorization, storage writes, and token transfer.
   */
  public static readonly AVG_BUY_ACCESS_FEE_XLM = 0.005;

  /**
   * Average gas fee for a single 'heartbeat' transaction in XLM.
   * Heartbeats are generally lighter than buy_access.
   */
  public static readonly AVG_HEARTBEAT_FEE_XLM = 0.0015;

  /**
   * Default heartbeat interval set in the contract (300 seconds / 5 minutes).
   * Note: This can be queried from the contract using `get_heartbeat_interval`.
   */
  public static readonly DEFAULT_HEARTBEAT_INTERVAL_SECONDS = 300;

  /**
   * Estimates the total XLM gas fee for a course session.
   * 
   * @param durationHours The duration of the course in hours.
   * @param heartbeatIntervalSeconds The interval between heartbeats in seconds.
   * @returns Total estimated XLM for gas.
   */
  public static estimateTotalGasXLM(
    durationHours: number,
    heartbeatIntervalSeconds: number = this.DEFAULT_HEARTBEAT_INTERVAL_SECONDS
  ): number {
    const durationSeconds = durationHours * 3600;
    const heartbeatCount = Math.ceil(durationSeconds / heartbeatIntervalSeconds);
    
    // Total = Buy Access Fee + (Number of Heartbeats * Heartbeat Fee)
    const baseTotal = this.AVG_BUY_ACCESS_FEE_XLM + (heartbeatCount * this.AVG_HEARTBEAT_FEE_XLM);
    
    // Add a 10% safety margin for network fluctuations and resource usage jitter
    return Number((baseTotal * 1.1).toFixed(6));
  }

  /**
   * Returns a detailed breakdown of the estimated gas costs.
   * Useful for showing students a transparent view of where their XLM is going.
   * 
   * @param durationHours The duration of the course in hours.
   * @param heartbeatIntervalSeconds The interval between heartbeats in seconds (optional).
   */
  public static getGasBreakdown(
    durationHours: number,
    heartbeatIntervalSeconds: number = this.DEFAULT_HEARTBEAT_INTERVAL_SECONDS
  ) {
    const durationSeconds = durationHours * 3600;
    const heartbeats = Math.ceil(durationSeconds / heartbeatIntervalSeconds);
    const baseFeeTotal = this.AVG_BUY_ACCESS_FEE_XLM + (heartbeats * this.AVG_HEARTBEAT_FEE_XLM);
    
    return {
      durationHours,
      buyAccessFeeXLM: this.AVG_BUY_ACCESS_FEE_XLM,
      heartbeatInterval: heartbeatIntervalSeconds,
      totalHeartbeats: heartbeats,
      totalHeartbeatFeesXLM: Number((heartbeats * this.AVG_HEARTBEAT_FEE_XLM).toFixed(6)),
      safetyBufferXLM: Number((baseFeeTotal * 0.1).toFixed(6)),
      totalEstimatedXLM: this.estimateTotalGasXLM(durationHours, heartbeatIntervalSeconds)
    };
  }

  /**
   * Generates a student-friendly message explaining the gas requirement.
   * 
   * @param durationHours The duration of the course in hours.
   */
  public static getStudentFriendlyMessage(durationHours: number): string {
    const estimate = this.estimateTotalGasXLM(durationHours);
    return `To watch this ${durationHours}-hour course, you will need approximately ${estimate} XLM available in your wallet to cover network fees.`;
  }
}

/**
 * A simple React-ready hook for Gas Estimation (template).
 * Since this is a service file, we provide it as a separate export.
 */
export function useGasEstimation(durationHours: number, heartbeatInterval?: number) {
  const breakdown = GasEstimationService.getGasBreakdown(durationHours, heartbeatInterval);
  const message = GasEstimationService.getStudentFriendlyMessage(durationHours);
  
  return {
    ...breakdown,
    message,
    refresh: () => GasEstimationService.getGasBreakdown(durationHours, heartbeatInterval)
  };
}
