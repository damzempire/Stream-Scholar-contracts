export type StartStreamParams = {
  student: string;
  courseId: number;
  amount: string | number;
  token: string;
};

export type StopStreamParams = {
  student: string;
  courseId: number;
};

export type HeartbeatParams = {
  student: string;
  courseId: number;
  signature: string;
};

type ContractInvokeArgs = Record<string, unknown>;

type ContractClient = {
  invoke: (method: string, args: ContractInvokeArgs) => Promise<unknown>;
};

export class StreamScholarSDK {
  private client: ContractClient;

  constructor(client: ContractClient) {
    this.client = client;
  }

  async startStream(params: StartStreamParams) {
    const { student, courseId, amount, token } = params;

    if (!student) throw new Error("student is required");
    if (courseId === undefined || courseId === null) throw new Error("courseId is required");
    if (amount === undefined || amount === null || amount === "") throw new Error("amount is required");
    if (!token) throw new Error("token is required");

    try {
      return await this.client.invoke("buy_access", {
        student,
        course_id: courseId,
        amount,
        token,
      });
    } catch (error) {
      throw new Error(
        `startStream failed: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  async stopStream(params: StopStreamParams) {
    const { student, courseId } = params;

    if (!student) throw new Error("student is required");
    if (courseId === undefined || courseId === null) throw new Error("courseId is required");

    try {
      return await this.client.invoke("pro_rated_refund", {
        student,
        course_id: courseId,
      });
    } catch (error) {
      throw new Error(
        `stopStream failed: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  async sendHeartbeat(params: HeartbeatParams) {
    const { student, courseId, signature } = params;

    if (!student) throw new Error("student is required");
    if (courseId === undefined || courseId === null) throw new Error("courseId is required");
    if (!signature) throw new Error("signature is required");

    try {
      return await this.client.invoke("heartbeat", {
        student,
        course_id: courseId,
        _signature: signature,
      });
    } catch (error) {
      throw new Error(
        `sendHeartbeat failed: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }
}